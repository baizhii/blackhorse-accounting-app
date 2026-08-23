import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Expense, MonthSummary, NewExpensePayload } from "../types";

/** 当前月份 "YYYY-MM" */
function defaultMonth(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

export const useExpensesStore = defineStore("expenses", () => {
  const month = ref(defaultMonth());
  const items = ref<Expense[]>([]);
  const totalCents = ref(0);
  const count = ref(0);
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      const res = await invoke<MonthSummary>("list_expenses", { month: month.value });
      items.value = res.items;
      totalCents.value = res.totalCents;
      count.value = res.count;
    } finally {
      loading.value = false;
    }
  }

  async function add(payload: NewExpensePayload) {
    await invoke("create_expense", { payload });
    await load();
  }

  async function update(id: number, payload: NewExpensePayload) {
    await invoke("update_expense", { id, payload });
    await load();
  }

  async function remove(id: number) {
    await invoke("delete_expense", { id });
    await load();
  }

  return { month, items, totalCents, count, loading, load, add, update, remove };
});
