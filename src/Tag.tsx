// Tag.tsx
import React from 'react';

interface TagProps {
  title: string;
  colorClass: string;
}

const Tag: React.FC<TagProps> = ({ title, colorClass }) => {
  return (
    <div className={`tag ${colorClass}`}>
      {title}
    </div>
  );
};

export default Tag;