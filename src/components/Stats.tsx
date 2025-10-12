import dayjs from 'dayjs';
import { useEffect, useState } from 'react';
import { getYearStatistics, StatisticsData } from '../api/database';

import './Stats.css';

export default function Stats() {
  const [statsData, setStatsData] = useState<StatisticsData[]>([]);
  const today = dayjs();
  const startDate = dayjs('2025-01-01');
  const currentYear = today.year();

  useEffect(() => {
    const fetchYearStatistics = async () => {
      try {
        const data = await getYearStatistics(currentYear);
        setStatsData(data);
      } catch (error) {
        console.error('Failed to fetch year statistics:', error);
      }
    }
    fetchYearStatistics();
  }, [currentYear])

  const generateHeatmapData = () => {
    const weeks = [];
    let currentDate = startDate;

    while (currentDate.isBefore(today) || currentDate.isSame(today, 'day')) {
      const week = [];
      for (let i = 0; i < 7; i++) {
        const dateStr = currentDate.format('YYYY-MM-DD');
        const data = statsData.find(item => item.date === dateStr);

        week.push({
          date: dateStr,
          count: data?.count || 0,
        });

        currentDate = currentDate.add(1, 'day');
        if (currentDate.isAfter(today, 'day'))
          break;
      }
      weeks.push(week);
      if (currentDate.isAfter(today, 'day'))
        break;
    }
    return weeks;
  }

  const heatmapData = generateHeatmapData();

  return (
    <>
      <div className="heatmap-container">
        <div className="contribution-grid">
          {heatmapData.map((week, weekIndex) => (
            <div key={weekIndex} className='contribution-week'>
              {week.map((day) => (
                <div key={day.date}
                  className={`contribution-day level-${Math.min(Math.floor(day.count / 25000), 5)}`}
                >
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    </>
  );
};