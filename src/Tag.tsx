// Tag.tsx
import React from 'react';

interface TagProps {
  title: string;
  colorClass: string;
  onClick: (tag: string) => void;
}

const Tag: React.FC<TagProps> = ({ title, colorClass, onClick }) => {
  return (
    <div className={`tag ${colorClass}`} onClick={() => onClick(title)}>
      {title}
    </div>
  );
};

export default Tag;