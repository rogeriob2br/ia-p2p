/**
 * SkipToContent Component
 * Feature: landing-page-documentation
 * 
 * Accessibility component that allows keyboard users to skip navigation.
 */

interface SkipToContentProps {
  targetId?: string;
  label?: string;
}

export function SkipToContent({ 
  targetId = 'main-content',
  label = 'Pular para o conteúdo principal'
}: SkipToContentProps) {
  const handleClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault();
    const target = document.getElementById(targetId);
    if (target) {
      target.focus();
      target.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <a
      href={`#${targetId}`}
      onClick={handleClick}
      className="
        sr-only focus:not-sr-only
        fixed top-4 left-4 z-[100]
        px-4 py-2 rounded-lg
        bg-[#3C35F2] text-white font-medium
        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#3C35F2]
        transition-all
      "
    >
      {label}
    </a>
  );
}
