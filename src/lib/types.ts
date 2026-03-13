export type ResultType =
  | "url"
  | "wifi"
  | "vcard"
  | "event"
  | "email"
  | "phone"
  | "sms"
  | "geo"
  | "text";

export interface ScanResult {
  image_path: string;
  source_type: "region" | "window";
}

export interface ActionDefinition {
  id: string;
  label: string;
  payload: string;
  requiresConfirmation: boolean;
  confirmationMessage?: string;
}

export interface ParsedQRContent {
  type: ResultType;
  raw: string;
  displayText: string;
  actions: ActionDefinition[];
  fields?: Record<string, string>;
  warnings?: string[];
}

export interface ScanEntry {
  id: string;
  scannedAt: string;
  result: string;
  resultType: ResultType;
  parsedData?: Record<string, string>;
  sourceType: "window" | "region";
  sourceName?: string;
  thumbnailFile?: string;
}

export interface DecodeResult {
  text: string;
}
