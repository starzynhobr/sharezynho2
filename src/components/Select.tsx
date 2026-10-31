import { useState, type KeyboardEvent } from 'react';
import { Check, ChevronDown } from 'lucide-react';

interface SelectProps {
  label: string;
  value: string;
  defaultLabel: string;
  options: string[];
  onChange: (value: string) => void;
}

/** Themed dropdown; an empty value means the default option. */
export const Select: React.FC<SelectProps> = ({ label, value, defaultLabel, options, onChange }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);

  const items = [{ value: '', label: defaultLabel }, ...options.map((o) => ({ value: o, label: o }))];
  const selected = items.find((item) => item.value === value) ?? items[0];

  const open = () => {
    setActiveIndex(Math.max(items.indexOf(selected), 0));
    setIsOpen(true);
  };

  const choose = (index: number) => {
    onChange(items[index].value);
    setIsOpen(false);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!isOpen) {
      if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(e.key)) {
        e.preventDefault();
        open();
      }
      return;
    }
    if (e.key === 'Escape' || e.key === 'Tab') {
      if (e.key === 'Escape') e.preventDefault();
      setIsOpen(false);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, items.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      choose(activeIndex);
    }
  };

  return (
    <div className="relative min-w-0">
      <span className="block text-xs font-semibold text-slate-300 mb-1.5">{label}</span>
      <button
        type="button"
        role="combobox"
        aria-label={label}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        data-select
        onClick={() => (isOpen ? setIsOpen(false) : open())}
        onBlur={() => setIsOpen(false)}
        onKeyDown={handleKeyDown}
        className={`w-full h-9 bg-root border rounded-xl pl-3 pr-2.5 text-xs text-white flex items-center justify-between gap-2 transition-colors cursor-pointer ${
          isOpen ? 'border-brand' : 'border-border-subtle hover:border-border-strong'
        }`}
      >
        <span className="truncate">{selected.label}</span>
        <ChevronDown
          className={`w-3.5 h-3.5 shrink-0 text-slate-400 transition-transform ${isOpen ? 'rotate-180' : ''}`}
        />
      </button>

      {isOpen && (
        <ul
          role="listbox"
          aria-label={label}
          className="absolute z-10 bottom-full mb-1.5 left-0 w-max min-w-full max-w-80 max-h-52 overflow-y-auto bg-card border border-border-strong rounded-xl p-1 shadow-2xl"
        >
          {items.map((item, index) => (
            <li
              key={item.value || 'default'}
              role="option"
              aria-selected={item === selected}
              title={item.label}
              onMouseEnter={() => setActiveIndex(index)}
              onMouseDown={(e) => {
                e.preventDefault();
                choose(index);
              }}
              className={`flex items-center justify-between gap-3 rounded-lg px-2.5 py-2 text-xs cursor-pointer ${
                index === activeIndex ? 'bg-control text-white' : 'text-slate-200'
              }`}
            >
              <span className="truncate">{item.label}</span>
              {item === selected && <Check className="w-3.5 h-3.5 shrink-0 text-brand" />}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};
