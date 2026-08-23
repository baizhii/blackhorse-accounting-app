<script setup lang="ts">
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { useCategoriesStore } from "../stores/categories";
import type { Category } from "../types";

const cats = useCategoriesStore();
const selectedParentId = ref<number | null>(null);

const selectedParent = computed(() =>
  selectedParentId.value === null
    ? null
    : cats.parents.find((p) => p.id === selectedParentId.value) ?? null
);
const selectedChildren = computed(() =>
  selectedParentId.value === null ? [] : cats.childrenOf(selectedParentId.value)
);

// ---------- 新增 / 重命名对话框（共用） ----------
type DialogMode = "add-parent" | "add-child" | "rename";
const dialog = ref({
  visible: false,
  mode: "add-parent" as DialogMode,
  id: 0,
  name: "",
});

function openAddParent() {
  dialog.value = { visible: true, mode: "add-parent", id: 0, name: "" };
}

function openAddChild() {
  if (selectedParentId.value === null) {
    return ElMessage.warning("请先在左侧选择一个一级大类");
  }
  dialog.value = { visible: true, mode: "add-child", id: selectedParentId.value, name: "" };
}

function openRename(cat: Category) {
  dialog.value = { visible: true, mode: "rename", id: cat.id, name: cat.name };
}

const dialogTitle = computed(() => {
  switch (dialog.value.mode) {
    case "add-parent":
      return "新增一级大类";
    case "add-child":
      return `在「${selectedParent.value?.name ?? ""}」下新增二级小类`;
    case "rename":
      return "重命名分类";
  }
});

async function submitDialog() {
  const name = dialog.value.name.trim();
  if (!name) {
    return ElMessage.warning("名称不能为空");
  }
  try {
    if (dialog.value.mode === "rename") {
      await invoke("rename_category", { id: dialog.value.id, name });
    } else {
      await invoke("add_category", {
        name,
        parentId: dialog.value.mode === "add-child" ? dialog.value.id : null,
      });
    }
    dialog.value.visible = false;
    await cats.reload();
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error(`操作失败：${e}`);
  }
}

async function removeCategory(cat: Category) {
  try {
    await ElMessageBox.confirm(`确定删除分类「${cat.name}」？`, "删除确认", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
  } catch {
    return; // 用户取消
  }
  try {
    await invoke("delete_category", { id: cat.id });
    await cats.reload();
    ElMessage.success("已删除");
  } catch (e) {
    ElMessage.error(`删除失败：${e}`);
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <h2 class="page-title">分类管理</h2>
      <div class="ops">
        <el-button @click="openAddParent">＋ 新增一级大类</el-button>
        <el-button type="primary" @click="openAddChild">＋ 新增二级小类</el-button>
      </div>
    </div>
    <p class="tip">提示：有账单的分类不能删除；删除一级大类前需先删光其下的二级小类。</p>

    <div class="layout">
      <aside class="parents">
        <div
          v-for="p in cats.parents"
          :key="p.id"
          class="parent-item"
          :class="{ active: p.id === selectedParentId }"
          @click="selectedParentId = p.id"
        >
          <span class="parent-name">{{ p.name }}</span>
          <span class="parent-count">{{ cats.childrenOf(p.id).length }}</span>
        </div>
      </aside>

      <section class="children">
        <template v-if="selectedParent">
          <div v-for="c in selectedChildren" :key="c.id" class="child-row">
            <span class="child-name">{{ c.name }}</span>
            <span class="child-ops">
              <el-button size="small" link type="primary" @click="openRename(c)">重命名</el-button>
              <el-button size="small" link type="danger" @click="removeCategory(c)">删除</el-button>
            </span>
          </div>
        </template>
        <div v-else class="empty">← 请选择左侧的一级大类</div>
      </section>
    </div>

    <el-dialog v-model="dialog.visible" :title="dialogTitle" width="380">
      <el-input
        v-model="dialog.name"
        maxlength="20"
        placeholder="分类名称"
        @keyup.enter="submitDialog"
      />
      <template #footer>
        <el-button @click="dialog.visible = false">取消</el-button>
        <el-button type="primary" @click="submitDialog">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.page-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
}

.tip {
  color: var(--muted);
  font-size: 12px;
  margin-bottom: 16px;
}

.layout {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.parents {
  width: 200px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 6px;
  flex-shrink: 0;
}

.parent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.parent-item:hover {
  background: var(--hover);
}

.parent-item.active {
  background: var(--active-bg);
  color: #409eff;
  font-weight: 500;
}

.parent-count {
  color: var(--muted);
  font-size: 12px;
}

.children {
  flex: 1;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 16px;
  min-height: 300px;
}

.child-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px;
  border-bottom: 1px solid var(--hover);
}

.child-row:last-child {
  border-bottom: none;
}

.child-name {
  font-size: 14px;
}

.empty {
  color: var(--muted);
  font-size: 13px;
  padding: 24px 0;
  text-align: center;
}
</style>
