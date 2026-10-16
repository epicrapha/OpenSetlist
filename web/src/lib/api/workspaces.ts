import { request } from './client';
import type { Workspace, WorkspaceCreated } from './types';

export const createWorkspace = (name: string): Promise<WorkspaceCreated> =>
	request<WorkspaceCreated>('/workspaces', { method: 'POST', body: { name } });

export const getWorkspace = (id: string): Promise<Workspace> =>
	request<Workspace>(`/workspaces/${encodeURIComponent(id)}`);
