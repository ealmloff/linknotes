import React from "react";
import "./App.css";
import TextEditor from './TextEditor';

const App: React.FC = () => {
  return (
    <div className="app-container">
      <TextEditor />
    </div>
  );
};

export default App;