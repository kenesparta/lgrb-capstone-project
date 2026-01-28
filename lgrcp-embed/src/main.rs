#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use microbit_bsp::{ble::MultiprotocolServiceLayer, Config, Microbit};
use trouble_host::prelude::*;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

// Button event types
#[derive(Clone, Copy)]
pub enum ButtonEvent {
    APressed,
    AReleased,
    BPressed,
    BReleased,
}

// Global channel for button events - Fixed with proper mutex type
static BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 10> = Channel::new();

// Connection state signal
static CONNECTION_STATE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

// GATT Server definition
#[gatt_server]
struct Server {
    battery_service: BatteryService,
    button_service: ButtonService,
}

// Battery service
#[gatt_service(uuid = service::BATTERY)]
struct BatteryService {
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Battery Level")]
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify)]
    level: u8,
}

pub mod button_uuids {
    // EF680800-9B35-4933-9B10-52FFA9740042
    pub const SERVICE: [u8; 16] = [
        0xEF, 0x68, 0x08, 0x00, 0x9B, 0x35, 0x49, 0x33, 0x9B, 0x10, 0x52, 0xFF, 0xA9, 0x74, 0x00,
        0x42,
    ];

    // EF680801-9B35-4933-9B10-52FFA9740042
    pub const BUTTON_STATE: [u8; 16] = [
        0xEF, 0x68, 0x08, 0x01, 0x9B, 0x35, 0x49, 0x33, 0x9B, 0x10, 0x52, 0xFF, 0xA9, 0x74, 0x00,
        0x42,
    ];
}

#[gatt_service(uuid = button_uuids::SERVICE)]
struct ButtonService {
    #[characteristic(uuid = button_uuids::BUTTON_STATE, notify)]
    button_state: u8,
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

#[embassy_executor::task]
async fn button_task(mut btn_a: microbit_bsp::Button, mut btn_b: microbit_bsp::Button) {
    let sender = BUTTON_CHANNEL.sender();

    loop {
        match select(btn_a.wait_for_low(), btn_b.wait_for_low()).await {
            Either::First(()) => {
                // Button A pressed
                info!("[button] Button A (LEFT) pressed!");
                sender.send(ButtonEvent::APressed).await;
                btn_a.wait_for_high().await;
                info!("[button] Button A released");
                sender.send(ButtonEvent::AReleased).await;
            }
            Either::Second(()) => {
                // Button B pressed
                info!("[button] Button B (RIGHT) pressed!");
                sender.send(ButtonEvent::BPressed).await;
                btn_b.wait_for_high().await;
                info!("[button] Button B released");
                sender.send(ButtonEvent::BReleased).await;
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = Microbit::new(Config::default());
    let (sdc, mpsl) = board
        .ble
        .init(board.timer0, board.rng)
        .expect("BLE Stack failed to initialize");

    // Spawn button task
    spawner.must_spawn(button_task(board.btn_a, board.btn_b));
    spawner.must_spawn(mpsl_task(mpsl));

    run(sdc).await;
}

pub async fn run<C>(controller: C)
where
    C: Controller,
{
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address = Address::random([0x42, 0x6A, 0xE3, 0x1E, 0x83, 0xE7]);
    info!("Our address = {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    info!("Starting advertising and GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "LGR-BLE",
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .expect("Failed to create a GATT server");

    let app_task = async {
        loop {
            match advertise("LGR-BLE", &mut peripheral, &server).await {
                Ok(conn) => {
                    CONNECTION_STATE.signal(true);
                    connection_task(&server, &conn).await;
                    CONNECTION_STATE.signal(false);
                }
                Err(e) => {
                    let e = defmt::Debug2Format(&e);
                    panic!("[adv] error: {:?}", e);
                }
            }
        }
    };
    select(ble_task(runner), app_task).await;
}

/// This is a background task required to run forever alongside any other BLE tasks.
async fn ble_task<C: Controller, P: PacketPool>(
    mut runner: Runner<'_, C, P>,
) -> Result<(), BleHostError<C::Error>> {
    runner.run().await
}

/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
async fn advertise<'a, 'b, C: Controller>(
    name: &'a str,
    peripheral: &mut Peripheral<'a, C, DefaultPacketPool>,
    server: &'b Server<'_>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[[0x0f, 0x18], [0x00, 0x08]]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

/// This function will handle the GATT events and button events
async fn connection_task<P: PacketPool>(server: &Server<'_>, conn: &GattConnection<'_, '_, P>) {
    let level = server.battery_service.level;
    let button_state = server.button_service.button_state;

    info!("Setting initial battery level to 100");
    unwrap!(level.set(server, &100));
    info!("Setting initial button state to 0");
    unwrap!(button_state.set(server, &0)); // Initial state: no button pressed

    info!("Connection established. Press buttons A or B to send events!");
    info!("Waiting for GATT events or button events...");

    let receiver = BUTTON_CHANNEL.receiver();

    loop {
        match select(conn.next(), receiver.receive()).await {
            Either::First(event) => {
                info!("[gatt] Received GATT event");
                match event {
                    GattConnectionEvent::Disconnected { reason } => {
                        info!("[gatt] disconnected: {:?}", reason);
                        break;
                    }
                    GattConnectionEvent::Gatt { event } => {
                        info!("[gatt] Processing GATT request");
                        match event.accept() {
                            Ok(reply) => {
                                info!("[gatt] Sending reply");
                                reply.send().await
                            }
                            Err(e) => warn!("[gatt] error sending response: {:?}", e),
                        }
                    }
                    _ => {
                        info!("[gatt] Another event");
                    }
                }
            }
            Either::Second(button_event) => {
                info!("[button] Processing button event");
                match button_event {
                    ButtonEvent::APressed => {
                        info!("[button] Button A pressed, setting state to 1");
                        unwrap!(button_state.set(server, &1));
                        info!("[button] Attempting to notify button A press");
                        if let Err(e) = button_state.notify(conn, &1).await {
                            warn!("[button] Failed to notify button A press: {:?}", e);
                        } else {
                            info!("[button] Successfully notified button A press");
                        }
                    }
                    ButtonEvent::AReleased => {
                        info!("[button] Button A released, setting state to 0");
                        unwrap!(button_state.set(server, &0));
                        if let Err(e) = button_state.notify(conn, &0).await {
                            warn!("[button] Failed to notify button A release: {:?}", e);
                        } else {
                            info!("[button] Successfully notified button A release");
                        }
                    }
                    ButtonEvent::BPressed => {
                        info!("[button] Button B pressed, setting state to 2");
                        unwrap!(button_state.set(server, &2));
                        if let Err(e) = button_state.notify(conn, &2).await {
                            warn!("[button] Failed to notify button B press: {:?}", e);
                        } else {
                            info!("[button] Successfully notified button B press");
                        }
                    }
                    ButtonEvent::BReleased => {
                        info!("[button] Button B released, setting state to 0");
                        unwrap!(button_state.set(server, &0));
                        if let Err(e) = button_state.notify(conn, &0).await {
                            warn!("[button] Failed to notify button B release: {:?}", e);
                        } else {
                            info!("[button] Successfully notified button B release");
                        }
                    }
                }
            }
        }
    }

    info!("[gatt] task finished");
}
