import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CascaderOption, Category } from "../types";

export const useCategoriesStore = defineStore("categories", () => {
  const categories = ref<Category[]>([]);
  const loaded = ref(false);
  const error = ref("");

  const parents = computed(() => categories.value.filter((c) => c.parentId === null));
  const childrenOf = (id: number) => categories.value.filter((c) => c.parentId === id);
  const nameOf = (id: number) => categories.value.find((c) => c.id === id)?.name ?? `#${id}`;
  const parentIdOf = (id: number) =>
    categories.value.find((c) => c.id === id)?.parentId ?? null;

  /** 两级完整名称，如 "餐饮食品 / 午餐" */
  const fullNameOf = (id: number) => {
    const cat = categories.value.find((c) => c.id === id);
    if (!cat) return `#${id}`;
    if (cat.parentId === null) return cat.name;
    return `${nameOf(cat.parentId)} / ${cat.name}`;
  };

  /** 分类级联选择器（记一笔 / 编辑账单）的选项树 */
  const cascaderOptions = computed<CascaderOption[]>(() =>
    parents.value.map((p) => ({
      id: p.id,
      name: p.name,
      children: childrenOf(p.id).map((c) => ({ id: c.id, name: c.name })),
    }))
  );

  async function load() {
    if (loaded.value) return;
    try {
      categories.value = await invoke<Category[]>("list_categories");
      loaded.value = true;
    } catch (e) {
      error.value = String(e);
    }
  }

  async function reload() {
    loaded.value = false;
    categories.value = [];
    error.value = "";
    await load();
  }

  return {
    categories,
    loaded,
    error,
    parents,
    childrenOf,
    nameOf,
    parentIdOf,
    fullNameOf,
    cascaderOptions,
    load,
    reload,
  };
});
