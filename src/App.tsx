//import { useState } from "react";
//import reactLogo from "./assets/react.svg";
//import { invoke } from "@tauri-apps/api/core";
import "./App.css";
//import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import TextEditor from './TextEditor';



const App: React.FC = () => {
  return (
    <div className="App">
      <h1>LinkedNotes Editor</h1>
      <TextEditor />
    </div>
  );
};

export default App;

