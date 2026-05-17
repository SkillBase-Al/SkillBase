import React from 'react';
import LoginPage from './pages/LoginPage';
import Dashboard from './pages/Dashboard';

function App() {
  const [authed, setAuthed] = React.useState(!!localStorage.getItem('admin_token'));

  if (!authed) {
    return <LoginPage onLogin={() => setAuthed(true)} />;
  }

  return <Dashboard />;
}

export default App;
