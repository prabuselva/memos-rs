import { useState, useEffect, useCallback } from 'react';
import { notebooksApi, notesApi, type Note, type Notebook } from '../lib/api';
import { useNoteStore } from '../hooks/useNoteStore';
import { NoteCard } from './NoteCard';

interface NotebookNode {
  notebook: Notebook;
  children: NotebookNode[];
  notes: Note[];
}

interface NotebookBrowserProps {
  onSelectNote?: (noteId: string) => void;
}

export const NotebookBrowser = ({ onSelectNote }: NotebookBrowserProps) => {
  const { selectedNotebook, setSelectedNotebook, sidebarMode, setSidebarMode, setError } = useNoteStore();
  const [notebooksTree, setNotebooksTree] = useState<NotebookNode[]>([]);
  const [currentPath, setCurrentPath] = useState<Notebook[]>([]);
  const [currentContents, setCurrentContents] = useState<{folder: NotebookNode | null, notes: Note[]} | null>(null);
  const [rootContents, setRootContents] = useState<{folder: NotebookNode | null, notes: Note[]} | null>(null);
  const [loading, setLoading] = useState(true);
  const [unsortedNotes, setUnsortedNotes] = useState<Note[]>([]);

  const fetchTree = useCallback(async () => {
    try {
      const [notebooksRes, notesRes] = await Promise.all([
        notebooksApi.getAll(),
        notesApi.getAll(),
      ]);
      
      const notebooks = notebooksRes.data;
      const allNotes = notesRes.data;
      
      const tree = buildTree(notebooks);
      setNotebooksTree(tree);
      
      const rootNotes = allNotes.filter(n => !n.notebook_id);
      setUnsortedNotes(rootNotes);
      
      const rootFolder: NotebookNode | null = tree.length > 0 ? {
        notebook: { id: 'root', name: 'Root', parent_id: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
        children: tree,
        notes: rootNotes
      } : null;
      
      setRootContents({
        folder: rootFolder,
        notes: rootFolder?.children.flatMap(c => c.notes) || []
      });
      
      setCurrentContents({
        folder: rootFolder,
        notes: rootFolder?.children.flatMap(c => c.notes) || []
      });
      
    } catch (err) {
      console.error('Failed to fetch notebooks:', err);
      setError('Failed to load notebooks');
    } finally {
      setLoading(false);
    }
  }, [setError]);

  const buildTree = (notebooks: Notebook[]): NotebookNode[] => {
    const notebookMap = new Map<string, NotebookNode>();
    
    notebooks.forEach(nb => {
      notebookMap.set(nb.id, {
        notebook: nb,
        children: [],
        notes: []
      });
    });

    const roots: NotebookNode[] = [];
    
    notebooks.forEach(nb => {
      const node = notebookMap.get(nb.id)!;
      if (nb.parent_id) {
        const parent = notebookMap.get(nb.parent_id);
        if (parent) {
          parent.children.push(node);
        }
      } else {
        roots.push(node);
      }
    });

    return roots;
  };

  const fetchNotesForFolder = async (notebookId: string | null): Promise<Note[]> => {
    try {
      if (notebookId && notebookId !== 'root') {
        const response = await notebooksApi.getNotes(notebookId);
        return response.data;
      } else {
        const allNotesResponse = await notesApi.getAll();
        return allNotesResponse.data.filter(n => !n.notebook_id);
      }
    } catch (err) {
      console.error('Failed to fetch notes:', err);
      return [];
    }
  };

  const navigateToFolder = useCallback(async (folderId: string) => {
    setLoading(true);
    setError(null);
    
    try {
      const folderNotebook = notebooksTree.find(n => n.notebook.id === folderId);
      
      if (folderNotebook) {
        const folderNotes = await fetchNotesForFolder(folderId);
        
        const updatedTree = updateTreeNotebooks(notebooksTree, folderId, folderNotes);
        setNotebooksTree(updatedTree);
        
        setCurrentPath([...currentPath, folderNotebook.notebook]);
        setCurrentContents({
          folder: updatedTree.find(n => n.notebook.id === folderId) || null,
          notes: folderNotes
        });
      }
    } catch (err) {
      console.error('Failed to navigate to folder:', err);
      setError('Failed to load folder contents');
    } finally {
      setLoading(false);
    }
  }, [notebooksTree, currentPath, setError]);

  const updateTreeNotebooks = (trees: NotebookNode[], folderId: string, notes: Note[]): NotebookNode[] => {
    return trees.map(tree => {
      if (tree.notebook.id === folderId) {
        return { ...tree, notes };
      }
      if (tree.children.length > 0) {
        return { ...tree, children: updateTreeNotebooks(tree.children, folderId, notes) };
      }
      return tree;
    });
  };

  const goBack = () => {
    if (currentPath.length > 0) {
      const newPath = currentPath.slice(0, -1);
      setCurrentPath(newPath);
      
      if (newPath.length === 0 && rootContents) {
        setCurrentContents(rootContents);
      } else {
        const lastFolder = newPath[newPath.length - 1];
        navigateToFolder(lastFolder.id);
      }
    }
  };
  void goBack();

  const handleSelectNotebook = (notebookId: string | null) => {
    setSelectedNotebook(notebookId);
  };

  const handleNoteClick = (noteId: string) => {
    onSelectNote?.(noteId);
  };

  const renderNotes = (notes: Note[]) => {
    if (notes.length === 0) {
      return (
        <div className="flex flex-col items-center justify-center py-8 text-gray-500">
          <p className="text-sm">No notes in this folder</p>
        </div>
      );
    }

    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 p-4">
        {notes.map((note) => (
          <NoteCard
            key={note.id}
            note={note}
            onClick={() => handleNoteClick(note.id)}
            isSelected={false}
          />
        ))}
      </div>
    );
  };

  const renderFolders = (folders: NotebookNode[]) => {
    if (folders.length === 0) {
      return null;
    }

    return (
      <div className="p-4">
        {folders.map((folder) => (
          <div
            key={folder.notebook.id}
            onClick={() => navigateToFolder(folder.notebook.id)}
            className="mb-2 cursor-pointer rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-blue-400 dark:hover:border-blue-500 hover:shadow-md transition-all duration-200"
          >
            <div className="flex items-center gap-3 p-3">
              <div className="text-blue-500">
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2-2z" />
                </svg>
              </div>
              <div className="flex-1">
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                  {folder.notebook.name}
                </h3>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {folder.notes.length} notes • {folder.children.length} subfolders
                </p>
              </div>
              <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </div>
          </div>
        ))}
      </div>
    );
  };

  useEffect(() => {
    fetchTree();
  }, [fetchTree]);

  if (loading) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-gray-500">
        Loading...
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-hidden bg-white dark:bg-gray-800">
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 flex-shrink-0 flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Notebooks</h2>
        <div className="flex gap-1">
          <button
            onClick={() => setSidebarMode('list')}
            className={`p-1.5 rounded-md transition-colors ${
              sidebarMode === 'list'
                ? 'bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-200'
                : 'text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-700'
            }`}
            title="List view"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
            </svg>
          </button>
          <button
            onClick={() => setSidebarMode('tree')}
            className={`p-1.5 rounded-md transition-colors ${
              sidebarMode === 'tree'
                ? 'bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-200'
                : 'text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-700'
            }`}
            title="Tree view"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {sidebarMode === 'list' ? (
          <>
            <div className="px-4 py-2">
              <button
                onClick={() => handleSelectNotebook(null)}
                className={`
                  w-full flex items-center justify-between px-3 py-2 rounded-md cursor-pointer transition-colors
                  ${selectedNotebook === null 
                    ? 'bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100' 
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300'}
                `}
              >
                <span className="text-sm font-medium">Unsorted</span>
                <span className="text-xs text-gray-500">{unsortedNotes.length}</span>
              </button>
            </div>
            
            {notebooksTree.map((notebook) => (
              <div
                key={notebook.notebook.id}
                onClick={() => handleSelectNotebook(notebook.notebook.id)}
                className={`
                  flex items-center justify-between px-4 py-2 cursor-pointer transition-colors
                  ${selectedNotebook === notebook.notebook.id 
                    ? 'bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100' 
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300'}
                `}
              >
                <span className="text-sm font-medium">{notebook.notebook.name}</span>
                <span className="text-xs text-gray-500">{notebook.notes.length}</span>
              </div>
            ))}
            
            {currentContents && renderNotes(currentContents.notes)}
          </>
        ) : (
          <>
            {currentContents ? (
              <>
                {currentContents.folder && renderFolders([currentContents.folder])}
                {renderNotes(currentContents.notes)}
              </>
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-gray-500 p-4">
                <p className="text-sm">No notebooks yet</p>
                <p className="text-xs mt-1">Create a notebook to get started</p>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};
