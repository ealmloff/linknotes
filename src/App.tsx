/**
 * The `App` component is a functional component in a TypeScript React application that renders a
 * `TextEditor` component within a `div` with the class name "app-container".
 * @returns The `App` component is being returned, which contains a `div` element with a class name of
 * "app-container" and the `TextEditor` component.
 */
import React from "react";
import "./App.css";
/* The line `import TextEditor from './TextEditor';` is importing the `TextEditor` component from a
file named `TextEditor.tsx` or `TextEditor.jsx` in the same directory as the `App` component. This
allows the `App` component to use the functionality and UI elements defined in the `TextEditor`
component within its own rendering. */
import TextEditor from './TextEditor';

const App: React.FC = () => {
  return (
    /* The code `<div className="app-container">
          <TextEditor />
        </div>` is creating a `div` element with a class name of "app-container" and rendering the
    `TextEditor` component within it. This structure is part of the `App` component in a TypeScript
    React application. The `TextEditor` component is being included within the `div` element,
    allowing it to be displayed within the `App` component's rendering. */
    <div className="app-container">
      <TextEditor />
    </div>
  );
};

export default App;