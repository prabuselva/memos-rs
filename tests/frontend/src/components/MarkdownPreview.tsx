import { memo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import remarkEmoji from 'remark-emoji';

interface MarkdownPreviewProps {
  content: string;
}

export const MarkdownPreview = memo(({ content }: MarkdownPreviewProps) => {
  if (!content) {
    return (
      <div className="p-8 text-center text-gray-500 dark:text-gray-400">
        <p>No content</p>
      </div>
    );
  }

  return (
    <div className="prose dark:prose-invert max-w-none w-full px-4 sm:px-6 md:px-8 py-2 sm:py-3" style={{ overflowX: 'auto', overflowWrap: 'anywhere', wordBreak: 'break-word', zIndex: 50 }}>
      <ReactMarkdown remarkPlugins={[remarkGfm, remarkEmoji]}>{content}</ReactMarkdown>
    </div>
  );
});

MarkdownPreview.displayName = 'MarkdownPreview';