import { useEffect, useRef, useState } from 'react'
import './App.css'

type MessageType = 'system' | 'received' | 'sent' | 'error'

type ChatMessage = {
  id: string
  text: string
  type: MessageType
  time: string
}

function App() {
  const [socket, setSocket] = useState<WebSocket | null>(null)
  const [input, setInput] = useState('')
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const messagesRef = useRef<HTMLDivElement | null>(null)

  const wsUrl = 'ws://localhost:3000/ws' // Replace with your server URL if different

  function addMessage(text: string, type: MessageType) {
    const time = new Date().toLocaleTimeString()
    setMessages((prev) => [
      ...prev,
      { id: crypto.randomUUID(), text, type, time },
    ])
  }

  function connectWebSocket() {
    // If already open, ignore
    if (socket && socket.readyState === WebSocket.OPEN) return

    // Close any existing socket before creating a new one
    if (socket) {
      try {
        socket.close()
      } catch {
        console.error('Error closing existing socket');
      }
    }

    const ws = new WebSocket(wsUrl)

    ws.addEventListener('open', () => {
      addMessage('Connected to server', 'system')
    })

    ws.addEventListener('message', (event) => {
      addMessage(String(event.data), 'received')
    })

    ws.addEventListener('close', () => {
      addMessage('Disconnected from server', 'system')
    })

    ws.addEventListener('error', () => {
      addMessage('Connection error', 'error')
    })

    setSocket(ws)
  }

  function disconnectWebSocket() {
    if (socket) {
      socket.close()
      setSocket(null)
    }
  }

  function sendMessage() {
    const message = input.trim()
    if (!message) return

    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(message)
      addMessage(message, 'sent')
      setInput('')
    } else {
      addMessage('Not connected to server', 'error')
    }
  }

  // Send via Enter key
  function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') {
      e.preventDefault()
      sendMessage()
    }
  }

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    const el = messagesRef.current
    if (el) {
      el.scrollTop = el.scrollHeight
    }
  }, [messages])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (socket) {
        try {
          socket.close()
        } catch {
          console.error('Error closing socket on unmount');
        }
      }
    }
    // We intentionally omit socket from deps; we only want this on unmount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const isOpen = socket?.readyState === WebSocket.OPEN
  const isConnecting = socket?.readyState === WebSocket.CONNECTING

  return (
    <div className="ws-container">
      <h1>WebSocket Client</h1>

      <div className="controls">
        <button onClick={connectWebSocket} disabled={isOpen || isConnecting}>
          {isConnecting ? 'Connecting…' : 'Connect'}
        </button>
        <button onClick={disconnectWebSocket} disabled={!socket}>
          Disconnect
        </button>
      </div>

      <div id="messages" ref={messagesRef} className="messages">
        {messages.map((m) => (
          <div key={m.id} className={m.type}>
            {m.time}: {m.text}
          </div>
        ))}
      </div>

      <div className="input-row">
        <input
          type="text"
          placeholder="Type a message..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={onKeyDown}
        />
        <button onClick={sendMessage} disabled={!isOpen}>
          Send
        </button>
      </div>
    </div>
  )
}

export default App