/**
 * TagsPanel.tsx
 * Brief Description:
 * This component renders a panel of clickable tags, displaying the most recent ones and allowing users to add custom tags. It handles user interaction for tag selection, custom tag input, and displays a dropdown menu for additional options.
 * Programmer's Name: Suhaan, Teja, Siddh, Trisha
 * Date Created: 10/31/2024
 * Date Revised: 12/05/2024
 * Revision Description: Added isColorBlindMode prop, updated tag color styling, and improved dropdown menu functionality.
 * 
 * Preconditions:
 * - `tags`: an array of tag objects, each containing a `name` (string) and a `manual` (boolean) property.
 * - `onTagClick`: a function to handle when a tag is clicked, passing the tag name as an argument.
 * - `onAddTag`: a function to handle adding a custom tag, passing the tag name as an argument.
 * - `isColorBlindMode`: a boolean to toggle color-blind mode, affecting tag color styling.
 * 
 * Postconditions:
 * - Displays a panel with up to the last 5 tags in the `tags` list. The remaining tags are accessible through a dropdown menu when clicked.
 * - Allows the user to add a custom tag by typing in an input field and pressing Enter.
 * 
 * Error and Exception Conditions:
 * - If invalid data is passed in the `tags` array (e.g., missing `name` or `manual` properties), the component may not render correctly or throw an error.
 * - If `onTagClick` or `onAddTag` are not functions, the component will throw an error when trying to invoke them.
 * 
 * Side Effects:
 * - Toggles the dropdown menu open and closed, adding/removing event listeners for outside clicks and key presses based on the menu state.
 * - Updates the displayed tags dynamically as the `tags` prop or the custom tag input changes.
 * 
 * Invariants:
 * - The component will always render the tags in the correct order (most recent first) and only display up to the last 5 tags.
 * - The `onTagClick` and `onAddTag` functions must be passed correctly to handle user actions properly.
 * 
 * Known Faults:
 * - No known faults at this time.
 * 
 */

/* The code snippet `import React, { useState, useEffect, useRef } from 'react';` is importing specific
hooks and components from the React library and a custom component from a file named 'Tag'. */
import React, { useState, useEffect, useRef } from 'react';
import Tag from './Tag';

/* The `interface TagsPanelProps` is defining the props that the `TagsPanel` component expects to
receive. Here's a breakdown of each prop: */
interface TagsPanelProps {
  onTagClick: (tag: string) => void; // Callback for tag click
  tags: { name: string, manual: boolean }[]; // Array of tag objects with name and manual properties
  onAddTag: (tag: string) => void; // Callback for adding a custom tag
  isColorBlindMode: boolean; // Add prop for color-blind mode
}

/* This code snippet defines the `TagsPanel` component in TypeScript React. Here's a breakdown of what
the code is doing: */
const TagsPanel: React.FC<TagsPanelProps> = ({ onTagClick, tags, onAddTag, isColorBlindMode }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false); // State to track menu open/close
  const [customTag, setCustomTag] = useState(''); // State to store custom tag input
  const menuRef = useRef<HTMLDivElement>(null); // Reference to the dropdown menu

  /**
   * The function `toggleMenu` toggles the state of `isMenuOpen` in a TypeScript React component.
   */
  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen); // Toggle the menu open/close state
  };

  /**
   * The handleClickOutside function checks if a click event occurred outside a specified menu element
   * and closes the menu if so.
   * @param {MouseEvent} event - The `event` parameter in the `handleClickOutside` function is of type
   * `MouseEvent`, which represents a mouse event that occurs when a user interacts with the webpage
   * using a mouse.
   */
  const handleClickOutside = (event: MouseEvent) => {
    if (menuRef.current && !menuRef.current.contains(event.target as Node)) { // Check if the click is outside the menu
      setIsMenuOpen(false); // Close the menu
    }
  };

  /**
   * The function `handleKeyDown` sets the state of `isMenuOpen` to false when the 'Escape' key is
   * pressed.
   * @param {KeyboardEvent} event - The `event` parameter in the `handleKeyDown` function is of type
   * `KeyboardEvent`, which represents an event that occurs when a key is pressed on the keyboard. In
   * this case, the function is checking if the key that was pressed is the 'Escape' key and then
   * setting the `is
   */
  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') { // Check if the 'Escape' key is pressed
      setIsMenuOpen(false); // Close the menu
    }
  };

  /* The `useEffect` hook in the `TagsPanel` component is responsible for managing event listeners
  based on the state of `isMenuOpen`. Here's a breakdown of what it does: */
  useEffect(() => {
    if (isMenuOpen) { // Check if the menu is open
      document.addEventListener('mousedown', handleClickOutside); // Add event listener for outside clicks
      document.addEventListener('keydown', handleKeyDown); // Add event listener for key presses
    } else {
      document.removeEventListener('mousedown', handleClickOutside); // Remove event listener for outside clicks
      document.removeEventListener('keydown', handleKeyDown); // Remove event listener for key presses
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside); // Cleanup: remove event listener for outside clicks
      document.removeEventListener('keydown', handleKeyDown); // Cleanup: remove event listener for key presses
    };
  }, [isMenuOpen]);

  /**
   * The function `handleCustomTagInput` updates the state with the value of an input field in a React
   * component.
   * @param event - The `event` parameter in the `handleCustomTagInput` function is of type
   * `React.ChangeEvent<HTMLInputElement>`. This means it is an event object that is triggered when the
   * value of an input element changes, specifically an input element of type `HTMLInputElement` in a
   * React component.
   */
  const handleCustomTagInput = (event: React.ChangeEvent<HTMLInputElement>) => {
    setCustomTag(event.target.value); // Update the custom tag input state
  };

  /**
   * The function `addCustomTag` is triggered by a key press event in a React input element, and it
   * adds a custom tag to a list when the Enter key is pressed and the custom tag is not empty.
   * @param event - The `event` parameter is a React KeyboardEvent that represents a keyboard event,
   * such as a key press, that occurs when interacting with an `<input>` element in a React component.
   * In this case, the function `addCustomTag` is triggered when the 'Enter' key is pressed in the
   */
  const addCustomTag = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter' && customTag.trim() !== '') { // Check if Enter key is pressed and tag is not empty
      onAddTag(customTag); // Call the onAddTag function with the custom tag
      setCustomTag(''); // Clear the custom tag input
      setIsMenuOpen(false); // Close the menu after adding the tag
    }
  };

  // Extract only manual tags and limit the displayed tags to the last 5 added
  const displayedTags = tags.slice(-5); // Only the last 5 tags are displayed
  const remainingTags = tags.slice(0, -5); // Older tags go into the dropdown menu

  return (
    <div className="tags-panel">
      <div className="tags-list">
        {displayedTags.map((tag, index) => (
          <Tag key={index} title={tag.name} colorClass={`tag-color-${isColorBlindMode ? (index % 7) + 11 : (index % 10) + 1} ${isColorBlindMode ? 'color-blind-text' : ''}`} onClick={() => onTagClick(tag.name)} />
        ))}
      </div>
      <div className="menu-dots" onClick={toggleMenu}>⋮</div>
      {isMenuOpen && (
        <div className="dropdown-menu" ref={menuRef}>
          <input
            type="text"
            className="custom-tag-input"
            placeholder="Add a custom tag"
            value={customTag}
            onChange={handleCustomTagInput}
            onKeyDown={addCustomTag}
          />
          {remainingTags.map((tag, index) => (
            <Tag key={index} title={tag.name} colorClass={`tag-color-${isColorBlindMode ? (index % 7) + 11 : (index % 10) + 1} ${isColorBlindMode ? 'color-blind-text' : ''}`} onClick={() => onTagClick(tag.name)} />
          ))}
        </div>
      )}
    </div>
  );
};

export default TagsPanel;