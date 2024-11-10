import React, { useState, useEffect, useRef } from 'react';
import Tag from './Tag';

const TagsPanel: React.FC = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
      setIsMenuOpen(false);
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      setIsMenuOpen(false);
    }
  };

  useEffect(() => {
    if (isMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isMenuOpen]);

  return (
    <div className="tags-panel">
      <div className="tags-list">
        <Tag title="Tag1" colorClass="tag-color-1" />
        <Tag title="Tag2" colorClass="tag-color-2" />
        <Tag title="Tag3" colorClass="tag-color-3" />
        <Tag title="Tag4" colorClass="tag-color-4" />
        <Tag title="Tag5" colorClass="tag-color-5" />
      </div>
      <div className="menu-dots" onClick={toggleMenu}>⋮</div>
      {isMenuOpen && (
        <div className="dropdown-menu" ref={menuRef}>
          <div className="dropdown-item">Option 1</div>
          <div className="dropdown-item">Option 2</div>
          <div className="dropdown-item">Option 3</div>
        </div>
      )}
    </div>
  );
};

export default TagsPanel;