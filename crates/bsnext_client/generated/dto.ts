/*
 Generated by typeshare 1.9.2
*/

export type RouteKindDTO = 
	| { kind: "Html", payload: {
	html: string;
}}
	| { kind: "Json", payload: {
	json_str: string;
}}
	| { kind: "Raw", payload: {
	raw: string;
}}
	| { kind: "Sse", payload: {
	sse: string;
}}
	| { kind: "Proxy", payload: {
	proxy: string;
}}
	| { kind: "Dir", payload: {
	dir: string;
}};

export interface RouteDTO {
	path: string;
	kind: RouteKindDTO;
}

export interface ServerDesc {
	routes: RouteDTO[];
	id: string;
}

export type IdentityDTO = 
	| { kind: "Both", payload: {
	name: string;
	bind_address: string;
}}
	| { kind: "Address", payload: {
	bind_address: string;
}}
	| { kind: "Named", payload: {
	name: string;
}};

export interface ServerDTO {
	id: string;
	identity: IdentityDTO;
	socket_addr: string;
}

export interface GetServersMessageResponse {
	servers: ServerDTO[];
}

export type ServerChange = 
	| { kind: "Stopped", payload: {
	bind_address: string;
}}
	| { kind: "Started", payload?: undefined }
	| { kind: "Patched", payload?: undefined };

export interface ServerChangeSetItem {
	identity: IdentityDTO;
	change: ServerChange;
}

export interface ServerChangeSet {
	items: ServerChangeSetItem[];
}

export interface ServersStarted {
	servers_resp: GetServersMessageResponse;
	changeset: ServerChangeSet;
}

export enum EventLevel {
	External = "BSLIVE_EXTERNAL",
}

export type ExternalEvents = 
	| { kind: "ServersStarted", payload: ServersStarted }
	| { kind: "Watching", payload: Watching }
	| { kind: "WatchingStopped", payload: StoppedWatching }
	| { kind: "FileChanged", payload: FileChanged }
	| { kind: "FilesChanged", payload: FilesChangedDTO }
	| { kind: "InputFileChanged", payload: FileChanged }
	| { kind: "InputAccepted", payload: InputAccepted }
	| { kind: "InputError", payload: InputErrorDTO };

export interface ExternalEvent {
	level: EventLevel;
	fields: ExternalEvents;
}

export interface InputAccepted {
	path: string;
}

export interface FileChanged {
	path: string;
}

export interface FilesChangedDTO {
	paths: string[];
}

export interface DebounceDTO {
	kind: string;
	ms: string;
}

export interface Watching {
	paths: string[];
	debounce: DebounceDTO;
}

export interface StoppedWatching {
	paths: string[];
}

export type InputErrorDTO = 
	| { kind: "MissingInputs", payload: string }
	| { kind: "InvalidInput", payload: string }
	| { kind: "NotFound", payload: string }
	| { kind: "InputWriteError", payload: string }
	| { kind: "PathError", payload: string }
	| { kind: "PortError", payload: string }
	| { kind: "DirError", payload: string }
	| { kind: "YamlError", payload: string }
	| { kind: "MarkdownError", payload: string }
	| { kind: "Io", payload: string }
	| { kind: "UnsupportedExtension", payload: string }
	| { kind: "MissingExtension", payload: string };

export type ClientEvent = 
	| { kind: "Change", payload: ChangeDTO };

export type ChangeDTO = 
	| { kind: "Fs", payload: {
	path: string;
	change_kind: ChangeKind;
}}
	| { kind: "FsMany", payload: ChangeDTO[] };

export enum ChangeKind {
	Changed = "Changed",
	Added = "Added",
	Removed = "Removed",
}

