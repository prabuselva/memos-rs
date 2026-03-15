# API Documentation

The application provides a RESTful API for all operations. All authenticated endpoints require a Bearer token in the `Authorization` header.

## Table of Contents

- [Authentication](#authentication)
- [Notes](#notes)
- [Import/Export](#importexport)
- [Examples](#examples)

---

## Authentication

All authenticated endpoints require a Bearer token in the `Authorization` header:
```
Authorization: Bearer YOUR_TOKEN
```

### Register

Register a new user account.

**Endpoint:** `POST /api/register`

**Request Body:**
```json
{
  "username": "john",
  "email": "john@example.com",
  "password": "securepassword123"
}
```

**Response (201 Created):**
```json
{
  "success": true,
  "message": "Registration successful",
  "user": {
    "id": "uuid",
    "username": "john",
    "email": "john@example.com",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "token": "jwt_token_here"
}
```

---

### Login

Login and get a session token.

**Endpoint:** `POST /api/login`

**Request Body:**
```json
{
  "username": "john",
  "password": "securepassword123"
}
```

**Response (200 OK):**
```json
{
  "token": "jwt_token_here",
  "user": {
    "id": "uuid",
    "username": "john",
    "email": "john@example.com",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "expires_at": "2024-01-08T00:00:00Z"
}
```

---

### Logout

Logout the current user.

**Endpoint:** `POST /api/logout`

---

### Refresh Token

Refresh the session token.

**Endpoint:** `POST /api/refresh`

---

### Get Current User

Get the current user profile.

**Endpoint:** `GET /api/me`

---

### Update Profile

Update user profile information.

**Endpoint:** `PUT /api/profile`

**Request Body:**
```json
{
  "current_password": "currentpassword",
  "new_password": "newpassword123"
}
```

---

### Request Password Reset

Request a password reset token.

**Endpoint:** `POST /api/request-password-reset`

**Request Body:**
```json
{
  "email": "john@example.com"
}
```

---

### Reset Password

Reset password with the reset token.

**Endpoint:** `POST /api/reset-password`

**Request Body:**
```json
{
  "token": "reset_token",
  "password": "newpassword123"
}
```

---

## Notes

### List Notes

List all notes for the current user.

**Endpoint:** `GET /api/notes`

**Query Parameters:**
- `limit` (optional): Number of notes to return (default: 100)
- `offset` (optional): Number of notes to skip (default: 0)

**Example:**
```
GET /api/notes?limit=10&offset=0
```

**Response (200 OK):**
```json
[
  {
    "id": "uuid",
    "title": "My Note",
    "content": "# Hello World",
    "content_html": "<h1>Hello World</h1>",
    "parent_id": null,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "is_favorite": false,
    "is_archived": false,
    "tags": ["example", "demo"],
    "metadata": {},
    "user_id": "uuid"
  }
]
```

---

### Create Note

Create a new note.

**Endpoint:** `POST /api/notes`

**Request Body:**
```json
{
  "title": "My Note",
  "content": "# Hello World\nThis is my first note.",
  "parent_id": null,
  "tags": ["example", "demo"],
  "is_favorite": false,
  "is_archived": false
}
```

**Response (201 Created):**
```json
{
  "id": "uuid",
  "title": "My Note",
  "content": "# Hello World",
  "content_html": "<h1>Hello World</h1>",
  "parent_id": null,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "is_favorite": false,
  "is_archived": false,
  "tags": ["example", "demo"],
  "metadata": {},
  "user_id": "uuid"
}
```

---

### Get Note

Get a note by ID.

**Endpoint:** `GET /api/notes/{id}`

**Response (200 OK):**
```json
{
  "id": "uuid",
  "title": "My Note",
  "content": "# Hello World",
  "content_html": "<h1>Hello World</h1>",
  "parent_id": null,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "is_favorite": false,
  "is_archived": false,
  "tags": ["example", "demo"],
  "metadata": {},
  "user_id": "uuid"
}
```

---

### Update Note

Update an existing note.

**Endpoint:** `PUT /api/notes/{id}`

**Request Body:**
```json
{
  "title": "Updated Title",
  "content": "# Updated Content",
  "parent_id": null,
  "tags": ["updated", "tags"],
  "is_favorite": true,
  "is_archived": false
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "title": "Updated Title",
  "content": "# Updated Content",
  "content_html": "<h1>Updated Content</h1>",
  "parent_id": null,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-02T00:00:00Z",
  "is_favorite": true,
  "is_archived": false,
  "tags": ["updated", "tags"],
  "metadata": {},
  "user_id": "uuid"
}
```

---

### Delete Note

Delete a note.

**Endpoint:** `DELETE /api/notes/{id}`

**Response:** `204 No Content`

---

### Search Notes

Search notes by title and content.

**Endpoint:** `GET /api/notes/search`

**Query Parameters:**
- `q`: Search query string

**Example:**
```
GET /api/notes/search?q=hello
```

**Response (200 OK):**
```json
[
  {
    "id": "uuid",
    "title": "Hello World",
    "content": "This is a hello world note",
    ...
  }
]
```

---

### Get Note Content

Get note content rendered as HTML.

**Endpoint:** `GET /api/notes/{id}/content`

**Response (200 OK):**
```html
<h1>Hello World</h1>
<p>This is my first note.</p>
```

---

## Import/Export

### Import Tomboy Notes

Import notes from Tomboy XML format.

**Endpoint:** `POST /api/import/tomboy`

**Request Body:**
```json
{
  "notes": [
    "<?xml version=\"1.0\"?><note><title>My Note</title><content>Content</content><tags><tag>example</tag></tags></note>"
  ]
}
```

**Response (200 OK):**
```json
{
  "imported": 1,
  "note_ids": ["uuid"]
}
```

---

### Import Tomboy File

Import Tomboy notes from a file upload.

**Endpoint:** `POST /api/import/tomboy/file`

**Content-Type:** `multipart/form-data`

**Response (200 OK):**
```json
{
  "imported": 1,
  "note_ids": ["uuid"]
}
```

---

### Import Tomboy Directory

Import Tomboy notes from a server directory.

**Endpoint:** `POST /api/import/tomboy/directory`

**Response (200 OK):**
```json
{
  "imported": 5,
  "note_ids": ["uuid1", "uuid2", ...]
}
```

---

### Export Tomboy

Export all notes as Tomboy XML.

**Endpoint:** `GET /api/export/tomboy`

**Response (200 OK):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<notes>
  <note version="0.1">
    <title>My Note</title>
    <content>Content</content>
    <tags>
      <tag>example</tag>
    </tags>
    <last-modified>2024-01-01T00:00:00Z</last-modified>
  </note>
</notes>
```

---

## Health Check

### Health

Check if the server is running.

**Endpoint:** `GET /health`

**Response (200 OK):**
```
OK - Routes loaded
```

---

## Examples

### Create a Note

```bash
curl -X POST http://localhost:3000/api/notes \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My Note",
    "content": "# Hello World\nThis is my first note.",
    "tags": ["example", "demo"]
  }'
```

### Search Notes

```bash
curl -X GET "http://localhost:3000/api/notes/search?q=hello" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Import Tomboy Notes

```bash
curl -X POST http://localhost:3000/api/import/tomboy \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "notes": [
      "<?xml version=\"1.0\"?><note><title>My Note</title><content>Content</content></note>"
    ]
  }'
```

### Export All Notes

```bash
curl -X GET http://localhost:3000/api/export/tomboy \
  -H "Authorization: Bearer YOUR_TOKEN"
```