import { memo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

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
    <div className="prose dark:prose-invert max-w-none w-full px-2 sm:px-3 py-2 sm:py-3" style={{ overflowX: 'auto', overflowWrap: 'anywhere', wordBreak: 'break-word' }}>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
});

MarkdownPreview.displayName = 'MarkdownPreview';