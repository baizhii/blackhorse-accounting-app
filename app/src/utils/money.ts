/**
 * 元（number，最多两位小数）→ 分（整数）
 * 金额上限 99999999.99 元，双精度下乘 100 再四舍五入无误差
 */
export function yuanToCents(yuan: number): number {
  return Math.round(yuan * 100);
}

/** 分（整数）→ 元（字符串，保留两位小数） */
export function centsToYuan(cents: number): string {
  return (cents / 100).toFixed(2);
}
