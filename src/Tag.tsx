/**
 * Tag.tsx
 * Brief Description: This component renders a clickable tag with a title and a color class. It allows users to interact with tags, triggering an action when clicked.
 * Programmer's Name: Suhaan, Trisha, Teja, Siddh
 * Date Created: 10/31/2024
 * Date Revised: 12/05/2024
 * Revision Description: Updated prop types and added TypeScript support.
 * 
 * Preconditions:
 * - The component expects the `title` prop to be a string that represents the tag's name.
 * - The `colorClass` prop should be a valid CSS class that applies the desired color styling to the tag.
 * - The `onClick` prop should be a function that handles the click event, taking the `title` of the tag as an argument.
 * 
 * Postconditions:
 * - On click, the `onClick` function is called with the `title` of the tag passed as an argument.
 * 
 * Error and Exception Conditions:
 * - If `title` or `colorClass` is not provided or is of an incorrect type, the component may not render correctly or throw an error.
 * - If `onClick` is not provided as a function, calling it will result in an error.
 * 
 * Side Effects:
 * - The `onClick` function triggers side effects based on the passed function (such as updating state or triggering other UI changes).
 * Invariants:
 * - The `title` and `colorClass` props must always be valid strings before rendering.
 * Known Faults:
 * - No known faults at this time.
 * 
*/
// Tag.tsx
import React from 'react';

/* The `interface TagProps` in the provided TypeScript code is defining a type for the props that the
`Tag` component expects to receive. It specifies that the `TagProps` object should have three
properties: */
interface TagProps {
  title: string;
  colorClass: string;
  onClick: (tag: string) => void;
}

/**
 * The Tag component is a functional React component that displays a tag with a specified title and
 * color, and triggers a function when clicked.
 * @param  - The `Tag` component is a functional component in React that takes in three props: `title`,
 * `colorClass`, and `onClick`. The `title` prop is the text content to be displayed inside the tag.
 * The `colorClass` prop is a CSS class name that will be applied to
 * @returns A React functional component named Tag is being returned. It renders a div element with the
 * class "tag" and an additional class specified by the colorClass prop. The onClick event handler is
 * set to call the onClick function with the title prop as an argument when the div is clicked. The
 * content of the div is the title prop.
 */
const Tag: React.FC<TagProps> = ({ title, colorClass, onClick }) => {
  /* The `return` statement in the `Tag` component is returning a JSX element, specifically a `div`
  element. Here's a breakdown of what the JSX code inside the `return` statement is doing: */
  return (
    <div className={`tag ${colorClass}`} onClick={() => onClick(title)}>
      {title}
    </div>
  );
};

export default Tag;