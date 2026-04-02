#!/usr/bin/env python3
"""
Wikipedia Import Script for Test Data Generation
Usage: python import_wikipedia.py <category> <count>
"""

import requests
import json
import sys

def search_wikipedia(query, limit=10):
    url = f"https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={query}&srlimit={limit}&format=json"
    response = requests.get(url)
    return response.json().get('query', {}).get('search', [])

def get_wikipedia_page(title):
    url = f"https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=&explaintext=&titles={title}&format=json"
    response = requests.get(url)
    data = response.json()
    pages = data.get('query', {}).get('pages', {})
    for page_id, page_data in pages.items():
        return page_data.get('extract', '')
    return ''

def main():
    category = sys.argv[1] if len(sys.argv) > 1 else "technology"
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 50
    
    print(f"Searching Wikipedia for '{category}'...")
    results = search_wikipedia(category, count * 2)
    
    notes = []
    for result in results[:count]:
        title = result['title']
        snippet = result['snippet']
        content = get_wikipedia_page(title)
        
        note = {
            "title": title,
            "content": content if content else snippet,
            "tags": [category, "wikipedia"],
            "user_id": "test-user"
        }
        notes.append(note)
        print(f"  - {title}")
    
    with open(f"wikipedia_{category}_notes.json", "w") as f:
        json.dump(notes, f, indent=2)
    
    print(f"\nSaved {len(notes)} notes to wikipedia_{category}_notes.json")

if __name__ == "__main__":
    main()
