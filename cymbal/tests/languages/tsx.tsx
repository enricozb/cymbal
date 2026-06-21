export type UserId = string;

export type API = {
  send(message: string): void;
  receive(): string;
};

export type Result<T> =
  | { ok: true; value: T }
  | { ok: false; error: string };

export interface Repository<T> {
  findById(id: string): Promise<T | null>;
  save(entity: T): Promise<void>;
  delete(id: string): Promise<void>;
}

export interface User {
  id: UserId;
  name: string;
  email: string;
}

export class UserService {
  private cache: Map<string, User> = new Map();
  readonly maxRetries: number = 3;

  constructor(private readonly repo: Repository<User>) {}

  async getUser(id: UserId): Promise<User | null> {
    return this.repo.findById(id);
  }

  async createUser(name: string, email: string): Promise<User> {
    const user: User = { id: crypto.randomUUID(), name, email };
    await this.repo.save(user);
    return user;
  }
}

export function formatUser(user: User): string {
  return `${user.name} <${user.email}>`;
}

const parseEmail = (raw: string): Result<string> => {
  return raw.includes("@")
    ? { ok: true, value: raw.trim() }
    : { ok: false, error: "invalid email" };
};

export const DEFAULT_PAGE_SIZE = 20;
