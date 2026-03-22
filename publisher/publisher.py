#!/usr/bin/env python3
import os
import sys
import argparse
import pickle
import json
from google_auth_oauthlib.flow import InstalledAppFlow
from google.auth.transport.requests import Request
from googleapiclient.discovery import build
from googleapiclient.http import MediaFileUpload

# SCOPES para YouTube Data API v3
SCOPES = ['https://www.googleapis.com/auth/youtube.upload']
CLIENT_SECRETS_FILE = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'client_secrets.json')
TOKEN_PICKLE_FILE = 'token.pickle'

def get_authenticated_service():
    creds = None
    if os.path.exists(TOKEN_PICKLE_FILE):
        with open(TOKEN_PICKLE_FILE, 'rb') as token:
            creds = pickle.load(token)
    
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            if not os.path.exists(CLIENT_SECRETS_FILE):
                print(f"❌ Error: {CLIENT_SECRETS_FILE} not found.")
                sys.exit(1)
            
            flow = InstalledAppFlow.from_client_secrets_file(CLIENT_SECRETS_FILE, SCOPES)
            creds = flow.run_local_server(port=0)
        
        with open(TOKEN_PICKLE_FILE, 'wb') as token:
            pickle.dump(creds, token)

    return build('youtube', 'v3', credentials=creds)

def upload_video(file_path, title, description, privacy, category_id="28"):
    youtube = get_authenticated_service()

    body = {
        'snippet': {
            'title': title,
            'description': description,
            'tags': ['Sentinel', 'AI', 'Rust', 'Research'],
            'categoryId': category_id
        },
        'status': {
            'privacyStatus': privacy,
            'selfDeclaredMadeForKids': False,
        }
    }

    print(f"🚀 Uploading {file_path}...")
    media = MediaFileUpload(file_path, chunksize=-1, resumable=True)
    request = youtube.videos().insert(part=','.join(body.keys()), body=body, media_body=media)

    response = None
    while response is None:
        status, response = request.next_chunk()
        if status:
            print(f"   Upload progress: {int(status.progress() * 100)}%")

    print(f"✅ Upload Complete! Video ID: {response.get('id')}")
    print(f"   URL: https://youtu.be/{response.get('id')}")

def main():
    parser = argparse.ArgumentParser(description='Sentinel Publisher (Python Engine)')
    parser.add_argument('--file', required=True, help='Path to video file')
    parser.add_argument('--title', help='Video Title')
    parser.add_argument('--description', help='Video Description')
    parser.add_argument('--privacy', default='private', choices=['public', 'private', 'unlisted'], help='Privacy Status')
    
    args = parser.parse_args()

    if not os.path.exists(args.file):
        print(f"❌ Error: File {args.file} not found.")
        sys.exit(1)

    # Auto-detect title from filename if missing
    title = args.title if args.title else os.path.splitext(os.path.basename(args.file))[0]
    description = args.description if args.description else "Uploaded via Sentinel Publisher"

    try:
        upload_video(args.file, title, description, args.privacy)
    except Exception as e:
        print(f"❌ An error occurred: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
