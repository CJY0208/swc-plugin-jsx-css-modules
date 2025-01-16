import React from 'react'
import './App.module.css'

function App() {
  return (
    <div className="container">
      <h1 className="title">SWC Plugin JSX CSS Modules Test</h1>
      <p className="text">This text should be styled using CSS Modules</p>
      <div className="card :global(theme-dark)">
        <p className="card-text">This is a card with mixed local and global styles</p>
      </div>
    </div>
  )
}

export default App
