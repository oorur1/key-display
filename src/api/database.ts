import { invoke } from '@tauri-apps/api/core'

export type StatisticsData = {
  date: string;
  count: number;
}

export async function getStatistics(date: string): Promise<number | null> {
  try {
    return await invoke<number | null>('get_statistics', { date });
  } catch (error) {
    console.error(`Failed to get statistics for ${date}:`, error);
    throw error;
  }
}

export async function getYearStatistics(year: number): Promise<StatisticsData[]> {
  try {
    const data = await invoke<Array<[string, number]>>('get_year_statistics', { year });
    return data.map(([date, count]) => ({ date, count }));
  } catch (error) {
    console.error(`Failed to get year statistics for ${year}:`, error);
    throw error;
  }
}

export async function updateStatistics(date: string, notesCount: number): Promise<void> {
  try {
    await invoke('update_statistics', { date, notesCount });
  } catch (error) {
    console.error(`Failed to update statistics for ${date}:`, error);
    throw error;
  }
}