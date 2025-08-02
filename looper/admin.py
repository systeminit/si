#!/usr/bin/env python3
"""
SI Admin Migration Report Script

This script generates a report of workspaces sorted by their last updated date,
including the number of open change sets and change set metadata.
"""

import asyncio
import base64
import json
import os
import pathlib
import re
import sys
import argparse
import csv
from datetime import datetime
from urllib.parse import quote
import aiohttp
from h11 import ERROR
import websockets
from typing import List, Dict, Any, Optional, Set


class SIAdminReporter:
    def __init__(self, bearer_token: str, sdf_api_url: str):
        self.bearer_token = bearer_token
        self.sdf_api_url = sdf_api_url.rstrip('/')
        self.session: Optional[aiohttp.ClientSession] = None
        self.websocket: Optional[websockets.WebSocketServerProtocol] = None
        self.migration_results = dict[str, dict[str, Any]]() # key: changeset_id, value: migration result
        self.pending_migrations = dict[str, list[str]]() # key: changeset_id, value: list of ConnectionMigrated results

    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={
                'Authorization': f'Bearer {self.bearer_token}',
                'Content-Type': 'application/json',
                'Cache-Control': 'no-cache',
                'User-Agent': 'si.git/admin-tests (support@systeminit.com)'
            },

        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.websocket:
            await self.websocket.close()
        if self.session:
            await self.session.close()

    async def list_all_workspaces(self, query: Optional[str] = None) -> List[Dict[str, Any]]:
        """List all workspaces for SI."""
        print(f"📋 Fetching all workspaces{f' matching {query}' if query else ''}...")
        
        workspaces_url = f"{self.sdf_api_url}/v2/admin/workspaces"
        workspaces_url += f"?query={quote(query)}" if query else ""
        
        async with self.session.get(workspaces_url) as response:
            response.raise_for_status()
            data = await response.json()
            workspaces = data.get('workspaces', [])
            
            print(f"✅ Found {len(workspaces)} workspaces")
            return workspaces
    
    async def list_change_sets_for_workspace(self, workspace_id: str) -> List[Dict[str, Any]]:
        """List all change sets for a given workspace."""
        try:
            change_sets_url = f"{self.sdf_api_url}/v2/admin/workspaces/{workspace_id}/change_sets"
            
            async with self.session.get(change_sets_url) as response:
                response.raise_for_status()
                data = await response.json()
                
                # The API returns changeSets as an object with IDs as keys
                change_sets_dict = data.get('changeSets', {})
                change_sets = list(change_sets_dict.values())
                
                return change_sets
                
        except Exception as e:
            print(f"❌ Failed to list change sets for workspace {workspace_id}: {e}")
            return []

    async def download_snapshot(self, workspace_id: str, changeset_id: str):
        """Download the snapshot for a specific change set."""
        snapshot_url = f"{self.sdf_api_url}/v2/admin/workspaces/{workspace_id}/change_sets/{changeset_id}/get_snapshot"
        try:
            # The response is application/octet-stream
            async with self.session.get(snapshot_url) as response:
                response.raise_for_status()
                # Read the response as bytes that can be uploaded later
                snapshot = base64.b64decode(await response.read())
                print(f"✅ Successfully downloaded snapshot for workspace {workspace_id}, change set {changeset_id}")
                return snapshot
        except Exception as e:
            print(f"❌ Failed to download snapshot for workspace {workspace_id}, change set {changeset_id}: {e}")
            raise

    async def upload_snapshot(self, workspace_id: str, changeset_id: str, snapshot: bytes):
        """Upload a snapshot for a specific change set."""
        print(f"📤 Uploading snapshot for workspace {workspace_id}, change set {changeset_id}...")
        upload_url = f"{self.sdf_api_url}/v2/admin/workspaces/{workspace_id}/change_sets/{changeset_id}/set_snapshot"
        # upload_url = f"http://localhost:50007/v2/admin/workspaces/{workspace_id}/change_sets/{changeset_id}/upload_snapshot"
        data = aiohttp.FormData(default_to_multipart=True)
        data.add_field('snapshot', snapshot, filename=f"{workspace_id}_{changeset_id}.snapshot", content_type='application/octet-stream')
        boundary = '----WebKitFormBoundarywiBIWjWR7osAkgFI'
        with aiohttp.MultipartWriter('form-data', boundary) as multipart:
            # custom headers...
            # writer.headers['User-Agent'] = '...'

            part = multipart.append(snapshot)
            part.set_content_disposition('form-data', name='snapshot', filename=f"{workspace_id}_{changeset_id}.snapshot")

            try:
                async with self.session.post(upload_url, data=multipart, headers={ 'Content-Type': f'multipart/form-data; boundary={boundary}' }) as response:
                    response.raise_for_status()
                    print(f"✅ Successfully uploaded snapshot for workspace {workspace_id}, change set {changeset_id}")

            except Exception as e:
                print(f"❌ Error uploading snapshot for workspace {workspace_id}, change set {changeset_id}: {e}")
                print(await response.read())
                raise

    async def setup_websocket(self) -> bool:
        """Set up WebSocket connection for real-time migration events."""
        try:
            # Convert HTTP URL to WebSocket URL and use the correct path
            ws_url = self.sdf_api_url.replace('http://', 'ws://').replace('https://', 'wss://')
            ws_url = f"{ws_url}/ws/workspace_updates?token=Bearer%20{quote(self.bearer_token)}"
            
            print(f"🔌 Connecting to WebSocket: {ws_url}")
            self.websocket = await websockets.connect(ws_url)
            print("✅ WebSocket connected successfully")
            return True
            
        except Exception as e:
            print(f"❌ Failed to connect to WebSocket: {e}")
            return False
    
    async def handle_websocket_message(self, message: str) -> None:
        """Handle incoming WebSocket messages for migration events."""
        try:
            data = json.loads(message)
            
            # Extract fields from the actual payload structure
            payload = data.get('payload', {})
            payload_kind = payload.get('kind', '')
            change_set_id = data.get('change_set_id', '')
            workspace_pk = data.get('workspace_pk', '')
            
            if payload_kind == 'ConnectionMigrationStarted':
                migration_data = payload.get('data', {})
                dry_run = migration_data.get('dryRun', False)
                
                print(f"🚀 Migration started - Workspace: {workspace_pk}, ChangeSet: {change_set_id}, DryRun: {dry_run}")
                
                if change_set_id:
                    self.pending_migrations[change_set_id] = []
                else:
                    print("⚠️ No changeset ID provided in migration start event", data)
                    
            elif payload_kind == 'ConnectionMigrationFinished':
                migration_data = payload.get('data', {})
                dry_run = migration_data.get('dryRun', False)
                connections = migration_data.get('connections', 0)
                migrated = migration_data.get('migrated', 0)
                unmigrateable = migration_data.get('unmigrateable', 0)
                
                print(f"✅ Migration finished - Workspace: {workspace_pk}, ChangeSet: {change_set_id}")
                print(f"   DryRun: {dry_run}, Connections: {connections}, Migrated: {migrated}, Unmigrateable: {unmigrateable}")
                
                migrations = self.pending_migrations.pop(change_set_id)
                if migrations is not None:
                    self.migration_results[change_set_id] = {
                        'workspace_pk': workspace_pk,
                        'dry_run': dry_run,
                        'connections': connections,
                        'migrated': migrated,
                        'unmigrateable': unmigrateable,
                        'timestamp': datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC'),
                        'success': True,
                        'migrations': migrations
                    }
                else:
                    print("⚠️ No pending migrations for changeset", change_set_id, ", ".join(self.pending_migrations.keys()))
                    
            elif payload_kind == 'ConnectionMigrated':
                migrations = self.pending_migrations.get(change_set_id)
                if migrations is not None:
                    migration = payload.get('data', {}).get('message', '<WSEVENT ERROR: NO MESSAGE>')
                    migrations.append(migration)
                else:
                    print("⚠️ No pending migrations for changeset", change_set_id)

        except json.JSONDecodeError:
            print(f"⚠️ Failed to parse WebSocket message: {message}")
        except Exception as e:
            print(f"❌ Error handling WebSocket message: {e}")
    
    async def listen_websocket(self) -> None:
        """Listen for WebSocket messages."""
        if not self.websocket:
            return
            
        try:
            async for message in self.websocket:
                await self.handle_websocket_message(message)
        except websockets.exceptions.ConnectionClosed:
            print("🔌 WebSocket connection closed")
        except Exception as e:
            print(f"❌ WebSocket listener error: {e}")

    async def trigger_migration(self, workspace_id: str, changeset_id: str) -> bool:
        """Trigger a dry run migration for a specific changeset."""
        migration_url = f"{self.sdf_api_url}/v2/admin/workspaces/{workspace_id}/change_sets/{changeset_id}/migrate_connections"

        async with self.session.post(migration_url) as response:
            print(response, await response.text())
            response.raise_for_status()

    async def trigger_dry_run_migration(self, workspace_id: str, changeset_id: str) -> bool:
        """Trigger a dry run migration for a specific changeset."""
        try:
            migration_url = f"{self.sdf_api_url}/v2/admin/workspaces/{workspace_id}/change_sets/{changeset_id}/migrate_connections"
            
                    
            async with self.session.get(migration_url) as response:
                if response.status == 200:
                    print(f"🎯 Triggered dry run migration for changeset {changeset_id}")
                    return True
                else:
                    error_text = await response.text()
                    print(f"❌ Failed to trigger migration for changeset {changeset_id}: {response.status} - {error_text}")
                    return False
                    
        except Exception as e:
            print(f"❌ Error triggering migration for changeset {changeset_id}: {e}")
            return False
    
    async def wait_for_migration_completion(self, changeset_ids: List[str], timeout_seconds: int = 300) -> dict[str, dict[str, Any]]:
        """Wait for all migrations in the batch to complete."""
        start_time = asyncio.get_event_loop().time()
        
        migration_results = {}
        while True:
            # Check if any of our changeset_ids are still pending
            for change_set_id in changeset_ids:
                if change_set_id in self.migration_results:
                    migration_results[change_set_id] = self.migration_results.pop(change_set_id)
            remaining = [cid for cid in changeset_ids if cid not in migration_results]
            
            if not remaining:
                return migration_results
            elif (asyncio.get_event_loop().time() - start_time) >= timeout_seconds:
                raise TimeoutError(f"⚠️ Timeout waiting for migrations to complete. Still pending: {remaining}")

            await asyncio.sleep(1)
    
    def update_csv_with_migration_results(self, csv_data: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Update CSV data with migration results."""
        updated_data = []
        
        for row in csv_data:
            changeset_id = row.get('changeset_id', '')
            
            if changeset_id and changeset_id in self.migration_results:
                result = self.migration_results[changeset_id]
                
                # Determine migration status based on results
                if result.get('success', False):
                    connections = result.get('connections', 0)
                    migrated = result.get('migrated', 0)
                    unmigrateable = result.get('unmigrateable', 0)
                    
                    if connections == 0:
                        status = 'no_connections'
                        notes = 'No connections to migrate'
                    elif unmigrateable > 0:
                        status = 'has_unmigrateable'
                        notes = f'Connections: {connections}, Migrated: {migrated}, Unmigrateable: {unmigrateable}'
                    else:
                        status = 'ready_to_migrate'
                        notes = f'Connections: {connections}, All migrateable'
                else:
                    status = 'migration_failed'
                    notes = 'Dry run migration failed'
                
                row['migration_status'] = status
                row['migration_notes'] = notes
                row['last_checked'] = result.get('timestamp', row.get('last_checked', ''))
            
            updated_data.append(row)
        
        return updated_data

    def parse_datetime(self, date_string: str) -> datetime:
        """Parse ISO datetime string to datetime object."""
        try:
            # Handle both with and without microseconds
            if '.' in date_string:
                return datetime.fromisoformat(date_string.replace('Z', '+00:00'))
            else:
                return datetime.fromisoformat(date_string.replace('Z', '+00:00'))
        except Exception:
            return datetime.min
    
    def count_open_change_sets(self, change_sets: List[Dict[str, Any]]) -> int:
        """Count change sets with 'Open' status."""
        return sum(1 for cs in change_sets if cs.get('status') == 'Open')
    
    def format_datetime(self, dt: datetime) -> str:
        """Format datetime for display."""
        if dt == datetime.min:
            return "Unknown"
        return dt.strftime("%Y-%m-%d %H:%M:%S UTC")
    
    def load_users(self, users_file: str) -> List[Dict[str, Any]]:
        """Load users from JSON file."""
        try:
            with open(users_file, 'r') as f:
                users = json.load(f)
            print(f"📋 Loaded {len(users)} users from {users_file}")
            return users
        except Exception as e:
            print(f"❌ Failed to load users from {users_file}: {e}")
            return []
    
    def load_existing_csv(self, csv_file: str) -> Dict[str, Dict[str, Any]]:
        """Load existing CSV data for progress tracking."""
        existing_data = {}
        if not os.path.exists(csv_file):
            return existing_data
            
        try:
            with open(csv_file, 'r', newline='', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                for row in reader:
                    # Use a combination of user_email, workspace_id, and changeset_id as key
                    key = f"{row.get('user_email', '')}|{row.get('workspace_id', '')}|{row.get('changeset_id', '')}"
                    existing_data[key] = row
            print(f"📋 Loaded {len(existing_data)} existing records from {csv_file}")
        except Exception as e:
            print(f"❌ Failed to load existing CSV {csv_file}: {e}")
            
        return existing_data
    
    def write_csv_data(self, csv_file: str, csv_data: List[Dict[str, Any]]) -> None:
        """Write CSV data to file."""
        if not csv_data:
            print("⚠️ No data to write to CSV")
            return
            
        fieldnames = [
            'user_email', 'user_first_name', 'user_last_name', 'user_signup_at',
            'workspace_id', 'workspace_name', 'workspace_created_at', 'workspace_updated_at',
            'changeset_id', 'changeset_name', 'changeset_status', 'changeset_created_at', 'changeset_updated_at',
            'migration_status', 'migration_notes', 'last_checked'
        ]
        
        try:
            with open(csv_file, 'w', newline='', encoding='utf-8') as f:
                writer = csv.DictWriter(f, fieldnames=fieldnames)
                writer.writeheader()
                writer.writerows(csv_data)
            print(f"✅ Wrote {len(csv_data)} records to {csv_file}")
        except Exception as e:
            print(f"❌ Failed to write CSV {csv_file}: {e}")
    
    async def generate_migration_csv(self, users_file: str, csv_file: str) -> None:
        """Generate CSV with user/workspace/changeset data for migration tracking."""
        print("📊 Generating Migration Tracking CSV")
        print("=" * 60)
        print()
        
        # Load users
        users = self.load_users(users_file)
        if not users:
            print("❌ No users loaded")
            return
        
        # Load existing CSV data for progress tracking
        existing_data = self.load_existing_csv(csv_file)
        
        csv_data = []
        current_timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')
        
        for i, user in enumerate(users, 1):
            user_email = user.get('email', '')
            user_first_name = user.get('firstName', '')
            user_last_name = user.get('lastName', '')
            user_signup_at = user.get('signupAt', '')
            
            print(f"[{i}/{len(users)}] Processing user: {user_email}")
            
            # Get workspaces for this user
            workspaces = await self.list_all_workspaces(user_email)
            
            if not workspaces:
                # User with no workspaces - still create a record
                row_key = f"{user_email}||"
                existing_row = existing_data.get(row_key, {})
                
                csv_data.append({
                    'user_email': user_email,
                    'user_first_name': user_first_name,
                    'user_last_name': user_last_name,
                    'user_signup_at': user_signup_at,
                    'workspace_id': '',
                    'workspace_name': '',
                    'workspace_created_at': '',
                    'workspace_updated_at': '',
                    'changeset_id': '',
                    'changeset_name': '',
                    'changeset_status': '',
                    'changeset_created_at': '',
                    'changeset_updated_at': '',
                    'migration_status': existing_row.get('migration_status', 'no_workspaces'),
                    'migration_notes': existing_row.get('migration_notes', 'User has no workspaces'),
                    'last_checked': current_timestamp
                })
                continue
            
            for workspace in workspaces:
                workspace_id = workspace.get('id', '')
                workspace_name = workspace.get('name', '')
                workspace_created_at = workspace.get('created_at', '')
                workspace_updated_at = workspace.get('updated_at', '')
                
                # Get change sets for this workspace
                change_sets = await self.list_change_sets_for_workspace(workspace_id)
                
                # Filter for open change sets
                open_change_sets = [cs for cs in change_sets if cs.get('status') == 'Open']
                
                if not open_change_sets:
                    # Workspace with no open change sets
                    row_key = f"{user_email}|{workspace_id}|"
                    existing_row = existing_data.get(row_key, {})
                    
                    csv_data.append({
                        'user_email': user_email,
                        'user_first_name': user_first_name,
                        'user_last_name': user_last_name,
                        'user_signup_at': user_signup_at,
                        'workspace_id': workspace_id,
                        'workspace_name': workspace_name,
                        'workspace_created_at': workspace_created_at,
                        'workspace_updated_at': workspace_updated_at,
                        'changeset_id': '',
                        'changeset_name': '',
                        'changeset_status': '',
                        'changeset_created_at': '',
                        'changeset_updated_at': '',
                        'migration_status': existing_row.get('migration_status', 'no_open_changesets'),
                        'migration_notes': existing_row.get('migration_notes', 'Workspace has no open change sets'),
                        'last_checked': current_timestamp
                    })
                else:
                    # Create row for each open change set
                    for changeset in open_change_sets:
                        changeset_id = changeset.get('id', '')
                        changeset_name = changeset.get('name', '')
                        changeset_status = changeset.get('status', '')
                        changeset_created_at = changeset.get('createdAt', '')
                        changeset_updated_at = changeset.get('updatedAt', '')
                        
                        row_key = f"{user_email}|{workspace_id}|{changeset_id}"
                        existing_row = existing_data.get(row_key, {})
                        
                        csv_data.append({
                            'user_email': user_email,
                            'user_first_name': user_first_name,
                            'user_last_name': user_last_name,
                            'user_signup_at': user_signup_at,
                            'workspace_id': workspace_id,
                            'workspace_name': workspace_name,
                            'workspace_created_at': workspace_created_at,
                            'workspace_updated_at': workspace_updated_at,
                            'changeset_id': changeset_id,
                            'changeset_name': changeset_name,
                            'changeset_status': changeset_status,
                            'changeset_created_at': changeset_created_at,
                            'changeset_updated_at': changeset_updated_at,
                            'migration_status': existing_row.get('migration_status', 'pending'),
                            'migration_notes': existing_row.get('migration_notes', ''),
                            'last_checked': current_timestamp
                        })
        
        # Write CSV data
        self.write_csv_data(csv_file, csv_data)
        
        # Print summary
        print(f"\n📊 MIGRATION CSV SUMMARY")
        print("-" * 30)
        print(f"Total users processed: {len(users)}")
        print(f"Total CSV records: {len(csv_data)}")
        users_with_workspaces = len(set(row['user_email'] for row in csv_data if row['workspace_id']))
        users_without_workspaces = len(users) - users_with_workspaces
        print(f"Users with workspaces: {users_with_workspaces}")
        print(f"Users without workspaces: {users_without_workspaces}")
        open_changesets = len([row for row in csv_data if row['changeset_id']])
        print(f"Open change sets to migrate: {open_changesets}")
        print(f"CSV file: {csv_file}")
        print("=" * 60)

    async def run_migrations(self, workspaces_file: Optional[str] = None) -> None:
        if not workspaces_file:
            raise ValueError("Must specify --workspaces-file")

        results_dir = pathlib.Path("results")
        results_dir.mkdir(exist_ok=True)

        workspace_ids = open(workspaces_file).read().splitlines()
        for workspace_id in workspace_ids:
            workspace = (await self.list_all_workspaces(workspace_id))[0]
            workspace_name = workspace['name']
            change_set_id = workspace['defaultChangeSetId']

            # Save the current snapshot
            snapshot = await self.download_snapshot(workspace_id, change_set_id)
            snapshot_filename = f"{results_dir}/{workspace_id}.{change_set_id}.snapshot"
            with open(snapshot_filename, 'xb') as f:
                f.write(snapshot)

            # Trigger the migration
            await self.trigger_migration(workspace_id, change_set_id)
            print(f"🎯 Triggered run migration for changeset {change_set_id} in workspace {workspace_id} ...")

            await asyncio.sleep(30)

    async def run_migration_dry_runs(self, csv_file: str, *, workspaces_file: Optional[str] = None, batch_size: int = 1, in_changeset: Optional[tuple[str, str]] = None) -> None:
        """Run dry run migrations for users in batches, testing only HEAD changesets."""
        print("🎯 Running Migration Dry Runs (User-based batches)")
        print("=" * 60)
        print()
        
        results_dir = pathlib.Path("results")
        results_dir.mkdir(exist_ok=True)

        workspaces_to_process = []
        if workspaces_file:
            for line in open(workspaces_file).readlines():
                workspace_id = line.rstrip()
                workspace = (await self.list_all_workspaces(workspace_id))[0]
                print(f"Got workspace {workspace_id}", workspace)
                workspaces_to_process.append({
                    'user_email': '',
                    'workspace_id': workspace_id,
                    'head_changeset': {
                        'changeset_id': workspace.get('defaultChangeSetId'),
                        'changeset_name': 'HEAD',
                        'workspace_name': workspace.get('name'),
                    },
                })
            
            csv_data = None

            print(f"Read {len(workspaces_to_process)} workspaces from {workspaces_file}")

        else:
            # Load existing CSV data
            existing_data = self.load_existing_csv(csv_file)
            if not existing_data:
                print("❌ No CSV data found. Run --csv-mode first to generate the CSV.")
                return
            
            # Convert to list for processing
            csv_data = list(existing_data.values())
            
            # Group data by user and workspace to find HEAD changesets
            user_workspaces = {}
            for row in csv_data:
                user_email = row.get('user_email', '')
                workspace_id = row.get('workspace_id', '')
                changeset_id = row.get('changeset_id', '')
                
                if user_email and workspace_id:
                    if user_email not in user_workspaces:
                        user_workspaces[user_email] = {}
                    if workspace_id not in user_workspaces[user_email]:
                        user_workspaces[user_email][workspace_id] = {
                            'default_changeset_id': None,
                            'workspace_data': None,
                            'changesets': []
                        }
                    
                    # Store workspace metadata
                    if row.get('workspace_name'):
                        user_workspaces[user_email][workspace_id]['workspace_data'] = row
                    
                    # Store changeset info
                    if changeset_id:
                        user_workspaces[user_email][workspace_id]['changesets'].append(row)
            
            # Find users with workspaces that need migration testing
            workspaces_to_process = []
            for user_email, workspaces in user_workspaces.items():
                for workspace_id, workspace_info in workspaces.items():
                    workspace_data = workspace_info['workspace_data']
                    if workspace_data:
                        # Get the default changeset ID from workspace data
                        default_changeset_id = workspace_data.get('workspace_id')  # This should be defaultChangeSetId
                        
                        # Find corresponding changeset row for the default changeset
                        # For now, we'll process the first open changeset as HEAD
                        open_changesets = [cs for cs in workspace_info['changesets'] if cs.get('changeset_status') == 'Open']
                        if open_changesets:
                            # Take the first open changeset as HEAD
                            head_changeset = open_changesets[0]
                            if head_changeset.get('migration_status') == 'pending':
                                workspaces_to_process.append({
                                    'user_email': user_email,
                                    'workspace_id': workspace_id,
                                    'head_changeset': head_changeset
                                })
        
        if not workspaces_to_process:
            print("✅ No users with pending HEAD changesets found for dry run migration.")
            return
        
        print(f"📋 Found {len(workspaces_to_process)} users with HEAD changesets for dry run migration")
        
        # Setup WebSocket connection
        if not await self.setup_websocket():
            print("❌ Failed to setup WebSocket connection. Cannot run migrations.")
            return
        
        # Start WebSocket listener task
        websocket_task = asyncio.create_task(self.listen_websocket())
        
        try:
            # Process users in batches
            total_batches = (len(workspaces_to_process) + batch_size - 1) // batch_size
            
            for batch_num in range(0, len(workspaces_to_process), batch_size):
                batch = workspaces_to_process[batch_num:batch_num + batch_size]
                batch_index = (batch_num // batch_size) + 1
                
                print(f"\n🚀 Processing user batch {batch_index}/{total_batches} ({len(batch)} users)")
                
                # Collect changeset IDs for this batch
                batch_changeset_ids = []
                
                # Process each user in the batch
                for user_info in batch:
                    user_email = user_info['user_email']
                    workspace_id = user_info['workspace_id']
                    head_changeset = user_info['head_changeset']
                    
                    changeset_id = head_changeset.get('changeset_id', '')
                    changeset_name = head_changeset.get('changeset_name', '')
                    workspace_name = head_changeset.get('workspace_name', '')
                    
                    print(f"  👤 User: {user_email}")
                    print(f"     🏢 Workspace: {workspace_name} ({workspace_id})")
                    print(f"     🎯 HEAD Changeset: {changeset_name} ({changeset_id})")
                    
                    if in_changeset:
                        assert batch_size == 1
                        [in_workspace_id, in_changeset_id] = in_changeset
                        snapshot = await self.download_snapshot(workspace_id, changeset_id)
                        await self.upload_snapshot(in_workspace_id, in_changeset_id, snapshot)
                    else:
                        in_workspace_id = workspace_id
                        in_changeset_id = changeset_id
                    success = await self.trigger_dry_run_migration(in_workspace_id, in_changeset_id)
                    if success:
                        batch_changeset_ids.append(in_changeset_id)
                    else:
                        # Mark as failed in results
                        self.migration_results[changeset_id] = {
                            'workspace_pk': workspace_id,
                            'dry_run': True,
                            'connections': 0,
                            'migrated': 0,
                            'unmigrateable': 0,
                            'timestamp': datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC'),
                            'success': False,
                            'migrations': ["<ERROR: Failed migration>"],
                        }
                
                # Wait for all migrations in this batch to complete
                if batch_changeset_ids:
                    print(f"  ⏳ Waiting for {len(batch_changeset_ids)} migrations to complete ({', '.join(batch_changeset_ids)})...")
                    migration_results = await self.wait_for_migration_completion(batch_changeset_ids, timeout_seconds=300)
                    print(f"✅ Batch migrations completed: {len(migration_results)} results received")
                    for changeset_id, results in migration_results.items():
                        with (results_dir / f"{workspace_name}.{workspace_id}.{changeset_id}.txt").open('w') as f:
                            for result in results.get('migrations', []):
                                f.write(f"{result}\n")

                # Add a small delay between batches
                # if batch_index < total_batches:
                    # print("  😴 Pausing between batches...")
                    # await asyncio.sleep(5)
            
            # Update CSV data with migration results
            if csv_data:
                print(f"\n📊 Updating CSV with migration results...")
                updated_csv_data = self.update_csv_with_migration_results(csv_data)
            
                # Write updated CSV
                self.write_csv_data(csv_file, updated_csv_data)
            
            # Print summary
            total_processed = len([cid for cid in self.migration_results.keys() if cid])
            successful_migrations = len([r for r in self.migration_results.values() if r.get('success', False)])
            
            print(f"\n📊 MIGRATION DRY RUN SUMMARY")
            print("-" * 30)
            print(f"Total workspaces processed: {len(workspaces_to_process)}")
            print(f"Total HEAD changesets processed: {total_processed}")
            print(f"Successful dry runs: {successful_migrations}")
            print(f"Failed dry runs: {total_processed - successful_migrations}")
            
            # Status breakdown
            status_counts = {}
            for row in updated_csv_data:
                if row.get('changeset_id') and row.get('changeset_id') in self.migration_results:
                    status = row.get('migration_status', 'unknown')
                    status_counts[status] = status_counts.get(status, 0) + 1
            
            print("\nStatus breakdown for processed changesets:")
            for status, count in sorted(status_counts.items()):
                print(f"  {status}: {count}")
            
            print("=" * 60)
            
        finally:
            # Clean up WebSocket task
            websocket_task.cancel()
            try:
                await websocket_task
            except asyncio.CancelledError:
                pass

    async def generate_workspace_report(self, workspaces_file: Optional[str] = None, output_file: Optional[str] = None, json_output: bool = False) -> None:
        """Generate a comprehensive workspace report sorted by last updated date."""
        print("📊 Generating SI Workspace Report")
        print("=" * 60)
        print()
        
        # Get all workspaces
        # Enhance workspace data with change set information
        enhanced_workspaces = []
        
        if workspaces_file:
            workspaces = {}
            workspace_ids = [line.rstrip() for line in open(workspaces_file).readlines()]
            print(f"Read {len(workspace_ids)} workspace IDs from {workspaces_file}")
        else:
            workspaces = { str(workspace['id']): workspace for workspace in await self.list_all_workspaces() }
            if not workspaces:
                print("❌ No workspaces found or failed to fetch workspaces")
                return
            workspace_ids = workspaces.keys()


        print("📈 Processing workspace data...")

        for workspace_id in workspace_ids:
            workspace = workspaces.get(workspace_id) or (await self.list_all_workspaces(workspace_id))[0]
            workspace_name = workspace['name']
            
            # Get change sets for this workspace
            change_sets = await self.list_change_sets_for_workspace(workspace_id)
            
            # Parse dates
            created_at = self.parse_datetime(workspace.get('created_at', ''))
            updated_at = self.parse_datetime(workspace.get('updated_at', ''))
            
            # Count open change sets
            open_change_sets_count = self.count_open_change_sets(change_sets)
            
            enhanced_workspace = {
                'id': workspace_id,
                'name': workspace_name,
                'created_at': created_at,
                'updated_at': updated_at,
                'defaultChangeSetId': workspace.get('defaultChangeSetId'),
                'snapshotVersion': workspace.get('snapshotVersion'),
                'componentConcurrencyLimit': workspace.get('componentConcurrencyLimit'),
                'total_change_sets': len(change_sets),
                'open_change_sets_count': open_change_sets_count,
                'change_sets': change_sets
            }
            
            enhanced_workspaces.append(enhanced_workspace)
        
        # Sort by updated_at (most recent first)
        enhanced_workspaces.sort(key=lambda w: w['updated_at'], reverse=True)
        
        # Calculate summary statistics
        total_change_sets = sum(ws['total_change_sets'] for ws in enhanced_workspaces)
        total_open_change_sets = sum(ws['open_change_sets_count'] for ws in enhanced_workspaces)
        high_activity_workspaces = [ws for ws in enhanced_workspaces if ws['open_change_sets_count'] > 5]
        
        # Prepare report data
        report_timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')
        
        if json_output:
            # Generate JSON report
            report_data = {
                "report_timestamp": report_timestamp,
                "summary": {
                    "total_workspaces": len(enhanced_workspaces),
                    "total_change_sets": total_change_sets,
                    "total_open_change_sets": total_open_change_sets,
                    "average_change_sets_per_workspace": total_change_sets/len(enhanced_workspaces) if enhanced_workspaces else 0,
                    "workspaces_with_open_change_sets": sum(1 for ws in enhanced_workspaces if ws['open_change_sets_count'] > 0),
                    "high_activity_workspaces_count": len(high_activity_workspaces)
                },
                "workspaces": []
            }
            
            for ws in enhanced_workspaces:
                workspace_data = {
                    "id": ws['id'],
                    "name": ws['name'],
                    "created_at": ws['created_at'].isoformat() if ws['created_at'] != datetime.min else None,
                    "updated_at": ws['updated_at'].isoformat() if ws['updated_at'] != datetime.min else None,
                    "defaultChangeSetId": ws['defaultChangeSetId'],
                    "snapshotVersion": ws['snapshotVersion'],
                    "componentConcurrencyLimit": ws['componentConcurrencyLimit'],
                    "total_change_sets": ws['total_change_sets'],
                    "open_change_sets_count": ws['open_change_sets_count'],
                    "change_sets": []
                }
                
                for cs in ws['change_sets']:
                    cs_data = {
                        "id": cs['id'],
                        "name": cs['name'],
                        "status": cs['status'],
                        "createdAt": cs.get('createdAt'),
                        "updatedAt": cs.get('updatedAt'),
                        "baseChangeSetId": cs.get('baseChangeSetId'),
                        "workspaceSnapshotAddress": cs.get('workspaceSnapshotAddress'),
                        "workspaceId": cs.get('workspaceId'),
                        "mergeRequestedByUserId": cs.get('mergeRequestedByUserId')
                    }
                    workspace_data["change_sets"].append(cs_data)
                
                report_data["workspaces"].append(workspace_data)
            
            # Write JSON to file or print
            json_output_str = json.dumps(report_data, indent=2, default=str)
            if output_file:
                with open(output_file, 'w') as f:
                    f.write(json_output_str)
                print(f"✅ JSON report saved to: {output_file}")
            else:
                print(json_output_str)
        else:
            # Generate text report
            report_lines = []
            report_lines.append("=" * 60)
            report_lines.append(f"📋 WORKSPACE MIGRATION REPORT ({report_timestamp})")
            report_lines.append("=" * 60)
            report_lines.append(f"Total Workspaces: {len(enhanced_workspaces)}")
            report_lines.append("")
            
            for i, ws in enumerate(enhanced_workspaces, 1):
                report_lines.append(f"{i:2d}. {ws['name']}")
                report_lines.append(f"    ID: {ws['id']}")
                report_lines.append(f"    Created:  {self.format_datetime(ws['created_at'])}")
                report_lines.append(f"    Updated:  {self.format_datetime(ws['updated_at'])}")
                report_lines.append(f"    Default Change Set: {ws['defaultChangeSetId']}")
                report_lines.append(f"    Snapshot Version: {ws['snapshotVersion']}")
                report_lines.append(f"    Component Limit: {ws['componentConcurrencyLimit']}")
                report_lines.append(f"    Change Sets: {ws['total_change_sets']} total, {ws['open_change_sets_count']} open")
                
                # Show open change sets details
                if ws['open_change_sets_count'] > 0:
                    report_lines.append(f"    Open Change Sets:")
                    open_change_sets = [cs for cs in ws['change_sets'] if cs.get('status') == 'Open']
                    for cs in open_change_sets[:5]:  # Show first 5 open change sets
                        cs_created = self.parse_datetime(cs.get('createdAt', ''))
                        cs_updated = self.parse_datetime(cs.get('updatedAt', ''))
                        report_lines.append(f"      - {cs['name']} ({cs['id']})")
                        report_lines.append(f"        Created: {self.format_datetime(cs_created)}")
                        report_lines.append(f"        Updated: {self.format_datetime(cs_updated)}")
                        if cs.get('baseChangeSetId'):
                            report_lines.append(f"        Base: {cs['baseChangeSetId']}")
                    if len(open_change_sets) > 5:
                        report_lines.append(f"      ... and {len(open_change_sets) - 5} more")
                
                report_lines.append("")
            
            # Summary statistics
            report_lines.append("📊 SUMMARY STATISTICS")
            report_lines.append("-" * 30)
            report_lines.append(f"Total Workspaces: {len(enhanced_workspaces)}")
            report_lines.append(f"Total Change Sets: {total_change_sets}")
            report_lines.append(f"Total Open Change Sets: {total_open_change_sets}")
            report_lines.append(f"Average Change Sets per Workspace: {total_change_sets/len(enhanced_workspaces):.1f}")
            report_lines.append(f"Workspaces with Open Change Sets: {sum(1 for ws in enhanced_workspaces if ws['open_change_sets_count'] > 0)}")
            
            # Show workspaces that need attention
            if high_activity_workspaces:
                report_lines.append("")
                report_lines.append("⚠️  WORKSPACES NEEDING ATTENTION (>5 open change sets):")
                for ws in high_activity_workspaces:
                    report_lines.append(f"  - {ws['name']}: {ws['open_change_sets_count']} open change sets")
            
            report_lines.append("")
            report_lines.append("=" * 60)
            
            # Write to file or print
            report_text = "\n".join(report_lines)
            if output_file:
                with open(output_file, 'w') as f:
                    f.write(report_text)
                print(f"✅ Report saved to: {output_file}")
            else:
                print(report_text)

async def main():
    """Main function to generate the workspace migration report."""
    
    # Parse command line arguments
    parser = argparse.ArgumentParser(
        description='Generate SI workspace migration report',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Print report to console
  BEARER_TOKEN=your_token python admin.py
  
  # Save text report to file
  BEARER_TOKEN=your_token python admin.py --output report.txt
  
  # Save JSON report to file
  BEARER_TOKEN=your_token python admin.py --output report.json --json
  
  # Generate CSV for migration tracking
  BEARER_TOKEN=your_token python admin.py --csv-mode
  
  # Generate CSV with custom files
  BEARER_TOKEN=your_token python admin.py --csv-mode --users-file my_users.json --csv-file tracking.csv
  
  # Run dry run migrations for pending changesets
  BEARER_TOKEN=your_token python admin.py --migrate
  
  # Run migrations with custom batch size (users per batch)
  BEARER_TOKEN=your_token python admin.py --migrate --batch-size 3
  
  # Use custom API URL
  BEARER_TOKEN=your_token SDF_API_URL=https://api.example.com python admin.py
        """
    )
    parser.add_argument(
        '--output', '-o',
        type=str,
        help='Output file path (if not specified, prints to console)'
    )
    parser.add_argument(
        '--json', '-j',
        action='store_true',
        help='Output in JSON format instead of text'
    )
    parser.add_argument(
        '--csv-mode', '-c',
        action='store_true',
        help='Generate CSV for migration tracking instead of report'
    )
    parser.add_argument(
        '--users-file', '-u',
        type=str,
        default='users.json',
        help='Path to users JSON file (default: users.json)'
    )
    parser.add_argument(
        '--workspaces-file', '-w',
        type=str,
        help='Path to list of workspaces to process'
    )
    parser.add_argument(
        '--csv-file',
        type=str,
        default='migration_tracking.csv',
        help='Path to CSV file for migration tracking (default: migration_tracking.csv)'
    )
    parser.add_argument(
        '--migrate',
        action='store_true',
        help='Run dry run migrations for pending changesets'
    )
    parser.add_argument(
        '--batch-size',
        type=int,
        default=1,
        help='Number of users to process in each batch (default: 1)'
    )
    parser.add_argument(
        '--in-workspace-id',
        type=str,
    )
    parser.add_argument(
        '--in-changeset-id',
        type=str,
    )
    
    args = parser.parse_args()
    
    # Get configuration from environment variables
    bearer_token = os.getenv('BEARER_TOKEN')
    sdf_api_url = os.getenv('SDF_API_URL', 'http://localhost:8080/api')
    
    if not bearer_token:
        print("❌ Error: BEARER_TOKEN environment variable is required")
        print("Usage: BEARER_TOKEN=your_token python admin.py")
        sys.exit(1)
    
    print(f"🔧 Configuration:")
    print(f"  - SDF API URL: {sdf_api_url}")
    print(f"  - Bearer Token: {'*' * len(bearer_token[:-4]) + bearer_token[-4:] if len(bearer_token) > 4 else '****'}")
    
    async with SIAdminReporter(bearer_token, sdf_api_url) as reporter:
        if args.csv_mode:
            # Generate CSV for migration tracking
            await reporter.generate_migration_csv(args.users_file, args.csv_file)
        elif args.migrate:
            # Run dry run migrations
            await reporter.run_migrations(workspaces_file=args.workspaces_file)
            # in_changeset = (args.in_workspace_id, args.in_changeset_id) if args.in_workspace_id and args.in_changeset_id else None
            # await reporter.run_migration_dry_runs(args.csv_file, workspaces_file=args.workspaces_file, batch_size=args.batch_size, in_changeset=in_changeset)
        else:
            # Generate the workspace report
            await reporter.generate_workspace_report(
                workspaces_file=args.workspaces_file,
                output_file=args.output,
                json_output=args.json
            )


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n⚠️ Report generation interrupted by user")
        sys.exit(0)
    # except Exception as e:
    #     print(f"\n❌ Unexpected error: {e}")
    #     sys.exit(1)