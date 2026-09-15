export type * from "./estree.js";

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}
