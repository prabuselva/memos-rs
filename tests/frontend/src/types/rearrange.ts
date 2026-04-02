export interface NotebookNode {
  id: string;
  name: string;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
  children: NotebookNode[];
  notes: Note[];
}

export interface Note {
  id: string;
  title: string;
  content: string;
  content_html?: string | null;
  notebook_id?: string | null;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
  is_favorite: boolean;
  is_archived: boolean;
  tags: string[];
  metadata: Record<string, any>;
  user_id?: string | null;
}

export interface ReorderItem {
  id: string;
  type: 'notebook' | 'note';
  parent_id?: string;
  order?: number;
}

export interface ReorderRequest {
  items: ReorderItem[];
}

export interface BulkDeleteRequest {
  ids: string[];
  type: 'notebook' | 'note';
}
