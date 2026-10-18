import type { ServerInfo } from "@/bindings/routes/server";
import type { UserSummaryResponse } from "@/bindings/routes/users";
import * as usersAPI from "@/lib/api/users";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import { failureText } from "@/lib/utils";

let page_size: number = 1;

export class UserSearch {
  users: UserSummaryResponse[] = $state([]);
  final_page: boolean = $state(true);
  pending: boolean = $state(false);
  error: string | null = $state(null);
  private username: string | undefined = undefined;

  async search(text: string | undefined): Promise<void> {
    if (this.pending) return;

    this.users = [];
    this.username = text;
    await this.loadMore();
  }

  async refresh(): Promise<void> {
    await this.search(this.username);
  }

  async loadMore(): Promise<void> {
    if (this.pending) return;

    this.pending = true;
    this.error = null;

    try {
      const page = await usersAPI.lookup({ username: this.username, after: this.users.at(-1)?.id });
      this.users.push(...page);
      this.final_page = page.length < page_size;
    } catch (err) {
      this.error = failureText(err);
    } finally {
      this.pending = false;
    }
  }
}

events.subscribe(LocalEvent.server_info_loaded, (info: ServerInfo) => {
  page_size = info.limit_users_page;
});
