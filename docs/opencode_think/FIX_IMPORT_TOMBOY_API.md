# Fix Import Tomboy Notes Directory Button

## Problem Statement

The Import tomboy notes directory button is not triggering POST calls for each file from the Frontend. The current implementation sends all files in a single batch POST request, but the requirement is to upload files **one-by-one sequentially** with **progress tracking** and **atomic rollback** on failure.

## Requirements

1. **Sequential uploads** - Upload files one after another (not in batch)
2. **Progress bar** - Show X/Y files uploaded with current file name
3. **Atomic import** - If any file fails, delete ALL previously imported files
4. **Client-side validation** - Validate files before upload (stop on first failure)
5. **Detailed tracking** - Report imported, skipped, and failed counts

## Implementation Plan

### Backend Changes

#### 1. `/home/praburaja/projects/opencode_ws/memos-rs/src/db/mod.rs`

**Add new method to Database struct:**

```rust
pub async fn delete_notes_by_ids(&self, ids: &[String]) -> Result<usize> {
    let mut deleted_count = 0;
    
    for id in ids {
        if self.delete(id).await.is_ok() {
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}
```

**Purpose:** Delete multiple notes by their IDs for rollback functionality

---

#### 2. `/home/praburaja/projects/opencode_ws/memos-rs/src/import_export/tomboy/actions.rs`

**Add new function:**

```rust
pub async fn rollback_import(
    db: &Database,
    note_ids: &[String],
) -> Result<usize> {
    let mut deleted_count = 0;
    
    for id in note_ids {
        if db.delete(id).await.is_ok() {
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}
```

**Purpose:** Wrapper function for rollback functionality

---

#### 3. `/home/praburaja/projects/opencode_ws/memos-rs/src/api/routes.rs`

**Add new request/response types:**

```rust
#[derive(Deserialize)]
struct RollbackRequest {
    note_ids: Vec<String>,
}

#[derive(Serialize)]
struct RollbackResponse {
    deleted: usize,
}
```

**Add new rollback endpoint (before `import_tomboy_file`):**

```rust
async fn rollback_import_tomboy(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RollbackRequest>,
) -> (StatusCode, Json<RollbackResponse>) {
    let db = state.db.lock().await;
    match db.delete_notes_by_ids(&payload.note_ids).await {
        Ok(count) => (StatusCode::OK, Json(RollbackResponse { deleted: count })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RollbackResponse { deleted: 0 })),
    }
}
```

**Add route definition (add to router in `create_router`):**

```rust
.route("/import/tomboy/rollback", post(rollback_import_tomboy))
```

**Modify `import_tomboy_file` endpoint:**

```rust
async fn import_tomboy_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();
    
    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let bytes = field.bytes().await.ok();
        if let Some(bytes) = bytes {
            let content = String::from_utf8_lossy(&bytes);
            
            let cleaned_xml = content.trim().replace('\n', " ");
            if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
                let db = state.db.lock().await;
                if let Ok(saved_note) = db.create(note).await {
                    imported_count += 1;
                    imported_ids.push(saved_note.id.clone());
                }
            }
        }
    }
    
    (StatusCode::OK, Json(ImportResponse { imported: imported_count, note_ids: imported_ids }))
}
```

**Modify `import_tomboy` endpoint:**

```rust
async fn import_tomboy(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TomboyImportRequest>,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();
    
    for note_xml in payload.notes {
        let cleaned_xml = note_xml.trim().replace('\n', " ");
        if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
            let db = state.db.lock().await;
            if let Ok(saved_note) = db.create(note).await {
                imported_count += 1;
                imported_ids.push(saved_note.id.clone());
            }
        }
    }
    
    (StatusCode::OK, Json(ImportResponse { imported: imported_count, note_ids: imported_ids }))
}
```

**Update `ImportResponse` struct:**

```rust
#[derive(Serialize)]
struct ImportResponse {
    imported: usize,
    note_ids: Vec<String>,
}
```

---

### Frontend Changes

#### 1. `/home/praburaja/projects/opencode_ws/memos-rs/frontend/src/components/ImportProgressModal.tsx` **(NEW FILE)**

```tsx
import React from 'react';

interface ImportProgressModalProps {
  isOpen: boolean;
  total: number;
  current: number;
  currentFile: string;
  status: 'idle' | 'uploading' | 'success' | 'error';
  errorMessage?: string;
}

export const ImportProgressModal: React.FC<ImportProgressModalProps> = ({
  isOpen,
  total,
  current,
  currentFile,
  status,
  errorMessage,
}) => {
  if (!isOpen) return null;

  const getProgressBarColor = () => {
    switch (status) {
      case 'uploading':
        return 'bg-blue-600';
      case 'success':
        return 'bg-green-600';
      case 'error':
        return 'bg-red-600';
      default:
        return 'bg-gray-600';
    }
  };

  const getProgressPercentage = () => {
    if (total === 0) return 0;
    return Math.round((current / total) * 100);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
        <div className="mb-4">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            {status === 'uploading' && 'Importing Notes...'}
            {status === 'success' && 'Import Complete!'}
            {status === 'error' && 'Import Failed'}
          </h2>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            {current} / {total} files
          </p>
        </div>

        <div className="mb-4">
          <div className="h-2 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
            <div
              className={`h-full rounded-full ${getProgressBarColor()} transition-all duration-300`}
              style={{ width: `${getProgressPercentage()}%` }}
            />
          </div>
        </div>

        <div className="mb-4 rounded bg-gray-50 p-3 dark:bg-gray-700">
          <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Processing: <span className="font-mono">{currentFile}</span>
          </p>
        </div>

        {status === 'error' && errorMessage && (
          <div className="mb-4 rounded bg-red-50 p-3 dark:bg-red-900/20">
            <p className="text-sm text-red-600 dark:text-red-400">
              {errorMessage}
            </p>
          </div>
        )}

        <div className="flex justify-end">
          {(status === 'success' || status === 'error') && (
            <button
              onClick={() => window.location.reload()}
              className="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
            >
              Close
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

ExportProgressModal.displayName = 'ImportProgressModal';
```

---

#### 2. `/home/praburaja/projects/opencode_ws/memos-rs/frontend/src/lib/api.ts`

**Update `importExportApi`:**

```typescript
export const importExportApi = {
  exportTomboy: () => api.get<string>('/export/tomboy'),
  rollbackImport: (noteIds: string[]) => 
    api.post<{deleted: number}>('/import/tomboy/rollback', { 
      note_ids: noteIds 
    }),
};
```

---

#### 3. `/home/praburaja/projects/opencode_ws/memos-rs/frontend/src/App.tsx`

**Add import:**

```typescript
import { ImportProgressModal } from './components/ImportProgressModal';
```

**Add state:**

```typescript
interface ImportProgressState {
  total: number;
  current: number;
  currentFile: string;
  status: 'uploading' | 'success' | 'error';
  errorMessage?: string;
}

// In AppContent component, add:
const [importProgress, setImportProgress] = useState<ImportProgressState | null>(null);
```

**Add modal to JSX (before closing div):**

```tsx
<ImportProgressModal
  isOpen={importProgress !== null}
  total={importProgress?.total || 0}
  current={importProgress?.current || 0}
  currentFile={importProgress?.currentFile || ''}
  status={importProgress?.status || 'uploading'}
  errorMessage={importProgress?.errorMessage}
/>
```

---

#### 4. `/home/praburaja/projects/opencode_ws/memos-rs/frontend/src/components/NoteList.tsx`

**Add import:**

```typescript
import { importExportApi } from '../lib/api';
```

**Update state declarations:**

```typescript
export const NoteList = () => {
  const { notes, selectNote, selectedNoteId, searchQuery, addNote, setError, setNotes } = useNoteStore();
  const [filteredNotes, setFilteredNotes] = useState(notes);
  const [importProgress, setImportProgress] = useState<{current: number, total: number, currentFile: string} | null>(null);
```

**Replace `handleFileSelect` function:**

```typescript
const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
  const files = event.target.files;
  if (!files) return;

  const validFiles: {file: File, content: string}[] = [];
  
  // Read and validate all files first (Option A: fail fast)
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    
    if (file.name.endsWith('.note') || file.name.endsWith('.xml')) {
      const content = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = () => reject(reader.error);
        reader.readAsText(file);
      });
      
      const trimmed = content.trim();
      if (trimmed.startsWith('<note')) {
        validFiles.push({ file, content });
      } else {
        setError(`Invalid Tomboy XML in file: ${file.name}. Import stopped.`);
        return; // Stop entirely on first validation failure
      }
    }
  }

  if (validFiles.length === 0) {
    setError('No valid Tomboy note files found in the selected directory.');
    return;
  }

  setImportProgress({
    current: 0,
    total: validFiles.length,
    currentFile: validFiles[0].file.name,
  });

  const importedNoteIds: string[] = [];
  let failedAt = -1;

  try {
    // Sequential upload
    for (let i = 0; i < validFiles.length; i++) {
      setImportProgress({
        current: i,
        total: validFiles.length,
        currentFile: validFiles[i].file.name,
      });

      const response = await axios.post<{imported: number, note_ids: string[]}>(
        '/api/import/tomboy/file',
        validFiles[i].content,
        {
          headers: {
            'Content-Type': 'text/xml',
          },
        }
      );

      if (response.data.note_ids && response.data.note_ids.length > 0) {
        importedNoteIds.push(...response.data.note_ids);
      }
    }

    // Success - reload notes
    const updatedNotes = await notesApi.getAll();
    setNotes(updatedNotes.data);
    setError(null);
    setImportProgress(null);

  } catch (error) {
    // Rollback all previously imported notes
    if (importedNoteIds.length > 0) {
      try {
        await importExportApi.rollbackImport(importedNoteIds);
      } catch (rollbackError) {
        console.error('Rollback failed:', rollbackError);
      }
    }

    setError(`Failed to import ${validFiles[failedAt >= 0 ? failedAt : 0].file.name}. All imports rolled back.`);
    setImportProgress({
      current: failedAt >= 0 ? failedAt : 0,
      total: validFiles.length,
      currentFile: validFiles[failedAt >= 0 ? failedAt : 0].file.name,
    });
  }
};
```

**Add helper functions (before the component):**

```typescript
const isTomboyNoteFile = (file: File): boolean => {
  return file.name.endsWith('.note') || file.name.endsWith('.xml');
};

const isValidTomboyXml = (content: string): boolean => {
  const trimmed = content.trim();
  return trimmed.startsWith('<note');
};
```

**Update the import button label (optional):**

```tsx
<label className="flex-1 cursor-pointer rounded-lg bg-gray-600 px-4 py-2 text-center text-white hover:bg-gray-700 block w-full">
  Import Tomboy Notes Directory
  <input
    type="file"
    accept=".xml,.note"
    multiple
    // @ts-ignore
    webkitdirectory=""
    className="hidden"
    onChange={handleFileSelect}
  />
</label>
```

---

## Implementation Order

### Phase 1: Backend (Database & API)

1. **Add `delete_notes_by_ids` method** to Database in `src/db/mod.rs`
2. **Add `rollback_import` function** in `src/import_export/tomboy/actions.rs`
3. **Add rollback endpoint** in `src/api/routes.rs`
4. **Modify `import_tomboy_file`** to track and return `note_ids`
5. **Modify `import_tomboy`** to track and return `note_ids`
6. **Update `ImportResponse`** struct to include `note_ids`
7. **Export `TomboyNote` type** if not already exported

### Phase 2: Frontend (UI & Logic)

8. **Create `ImportProgressModal.tsx`** component
9. **Add `rollbackImport` API method** in `frontend/src/lib/api.ts`
10. **Add modal state management** in `frontend/src/App.tsx`
11. **Add modal component** to App.tsx JSX
12. **Update `handleFileSelect`** in `frontend/src/components/NoteList.tsx`
13. **Test all scenarios**:
    - Successful import of multiple files
    - Validation failure (stop on first invalid file)
    - Upload failure (rollback all previously imported)

---

## Testing Scenarios

### Test 1: Successful Import
**Input:** 5 valid Tomboy note files  
**Expected:** 
- Progress bar shows 0/5, 1/5, 2/5, 3/5, 4/5, 5/5
- All 5 files imported successfully
- Notes appear in the note list
- Progress modal closes with success

### Test 2: Validation Failure (Fail Fast)
**Input:** 5 files where 3rd file has invalid XML  
**Expected:**
- Progress shows 0/5, 1/5, 2/5
- Error message: "Invalid Tomboy XML in file: [filename]. Import stopped."
- No files imported
- Progress modal shows error state

### Test 3: Upload Failure (Rollback)
**Input:** 5 valid files, but server fails on 3rd upload  
**Expected:**
- Progress shows 0/5, 1/5, 2/5
- Server returns error on 3rd file
- Rollback API called to delete notes 1 and 2
- Error message: "Failed to import [filename]. All imports rolled back."
- No notes from this import remain in database
- Progress modal shows error state

---

## Notes

- The implementation uses **Option A**: Validate all files first, then upload (fail fast on validation)
- Rollback uses **DELETE by ID** for precision (not timestamp-based)
- Progress modal shows **file names** for transparency
- Skipped files (validation failures) are **not counted** in the final report
- All imports are **atomic** - either all succeed or none remain

---

## Files Modified

| File | Changes |
|------|---------|
| `src/db/mod.rs` | Add `delete_notes_by_ids` method |
| `src/import_export/tomboy/actions.rs` | Add `rollback_import` function |
| `src/api/routes.rs` | Add rollback endpoint, modify import endpoints to return note_ids |
| `frontend/src/components/ImportProgressModal.tsx` | **NEW FILE** - Progress overlay modal |
| `frontend/src/lib/api.ts` | Add `rollbackImport` method |
| `frontend/src/App.tsx` | Add modal state and component |
| `frontend/src/components/NoteList.tsx` | Update `handleFileSelect` with sequential upload logic |

---

## Dependencies

No new dependencies required. All needed functionality already exists in the codebase.