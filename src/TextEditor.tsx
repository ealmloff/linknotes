import React, { useMemo, useState, useCallback } from 'react';
import { createEditor, Descendant } from 'slate';
import { Slate, Editable, withReact, RenderElementProps } from 'slate-react';
import { BaseEditor } from 'slate';
import { ReactEditor } from 'slate-react';

// Define custom types for the elements and text
type ParagraphElement = { type: 'paragraph'; children: CustomText[] };
type HeadingElement = { type: 'heading'; children: CustomText[] };
type CustomText = { text: string; bold?: boolean };

// Extend the Element type with custom element types
type CustomElement = ParagraphElement | HeadingElement;

// Extend the Editor type
type CustomEditor = BaseEditor & ReactEditor;

// Declare module augmentation for Slate to use custom types
declare module 'slate' {
  interface CustomTypes {
    Editor: CustomEditor;
    Element: CustomElement;
    Text: CustomText;
  }
}

const TextEditor: React.FC = () => {
  const editor = useMemo(() => withReact(createEditor()), []);

  // Define the initial value with a paragraph element
  const [value, setValue] = useState<Descendant[]>([
    {
      type: 'paragraph', // Custom element type
      children: [{ text: 'Write something here...' }],
    },
  ]);

  // Define the renderElement function with correct typing for props
  const renderElement = useCallback((props: RenderElementProps) => {
    switch (props.element.type) {
      case 'heading':
        return <h1 {...props.attributes}>{props.children}</h1>;
      case 'paragraph':
      default:
        return <p {...props.attributes}>{props.children}</p>;
    }
  }, []);

  return (
    <Slate
      editor={editor}
      initialValue={value} // Change `value` to `initialValue`
      onValueChange={(newValue) => setValue(newValue)} // Change `onChange` to `onValueChange`
    >
      <Editable renderElement={renderElement} />
    </Slate>
  );
};

export default TextEditor;
