<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { useCategoriesStore } from "../stores/categories";
import { useExpensesStore } from "../stores/expenses";
import { yuanToCents } from "../utils/money";

const cats = useCategoriesStore();
const exps = useExpensesStore();

const form = ref({
  amount: null as number | null,
  category: [] as number[],
  occurredAt: defaultNow(),
  note: "",
});
const submitting = ref(false);

/** 当前本地时间，格式 "YYYY-MM-DD HH:MM" */
function defaultNow(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

async function submit() {
  if (form.value.amount === null || form.value.amount <= 0) {
    return ElMessage.warning("请输入正确的金额");
  }
  if (form.value.category.length < 2) {
    return ElMessage.warning("请选择二级分类");
  }
  submitting.value = true;
  try {
    await exps.add({
      amountCents: yuanToCents(form.value.amount),
      categoryId: form.value.category[1],
      occurredAt: form.value.occurredAt,
      note: form.value.note.trim(),
    });
    ElMessage.success("已记录一笔 ✍️");
    // 金额与备注清空，分类与时间保留，方便连续记账
    form.value.amount = null;
    form.value.note = "";
  } catch (e) {
    ElMessage.error(`记录失败：${e}`);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <h2 class="page-title">记一笔</h2>
    <el-form class="record-form" label-position="top" @submit.prevent="submit">
      <el-form-item label="金额（元）" required>
        <el-input-number
          v-model="form.amount"
          :min="0.01"
          :max="99999999.99"
          :precision="2"
          :step="1"
          :controls="false"
          placeholder="0.00"
          class="amount-input"
        >
          <template #prefix>¥</template>
        </el-input-number>
      </el-form-item>

      <div class="row">
        <el-form-item label="分类" required class="grow">
          <el-cascader
            v-model="form.category"
            :options="cats.cascaderOptions"
            :props="{ value: 'id', label: 'name' }"
            placeholder="先选大类，再选小类"
            class="category-input"
          />
        </el-form-item>

        <el-form-item label="时间" class="grow">
          <el-date-picker
            v-model="form.occurredAt"
            type="datetime"
            format="YYYY-MM-DD HH:mm"
            value-format="YYYY-MM-DD HH:mm"
            placeholder="选择时间"
            class="time-input"
          />
        </el-form-item>
      </div>

      <el-form-item label="备注">
        <el-input
          v-model="form.note"
          maxlength="200"
          show-word-limit
          placeholder="可选，例如：和朋友聚餐"
        />
      </el-form-item>

      <el-button type="primary" size="large" :loading="submitting" class="submit-btn" @click="submit">
        保存这笔
      </el-button>
    </el-form>
  </div>
</template>

<style scoped>
.page {
  width: 100%;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 20px;
}

.record-form {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 24px 24px 8px;
}

.row {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.grow {
  flex: 1;
}

.amount-input,
.category-input,
.time-input {
  width: 100%;
}

/* 金额输入内容靠左显示（Element Plus 默认居中） */
.amount-input :deep(.el-input__inner) {
  text-align: left;
}

.submit-btn {
  width: 100%;
  margin: 8px 0 20px;
}
</style>
