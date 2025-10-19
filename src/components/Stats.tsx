import dayjs from 'dayjs';
import { useEffect, useState } from 'react';
import { getYearStatistics, saveCurrentCount, StatisticsData } from '../api/database';
import { IoMdRefresh } from 'react-icons/io';
import { MdNavigateBefore, MdNavigateNext } from 'react-icons/md';

import './Stats.css';

interface MonthData {
  monthName: string;
  weeks: StatisticsData[][];
}

export default function Stats() {
  const [statsData, setStatsData] = useState<StatisticsData[]>([]);
  const [heatmapData, setHeatmapData] = useState<MonthData[]>([]);
  const [currentYear, setCurrentYear] = useState(dayjs().year());
  const [isUpdating, setIsUpdating] = useState(false);

  const weekdays = ['', 'Mon', '', 'Wed', '', 'Fri', ''];

  const updateStatistics = async () => {
    setIsUpdating(true);
    try {
      await saveCurrentCount()
      const data = await getYearStatistics(currentYear);
      setStatsData(data);
    } catch (error) {
      console.error('Failed to upload statistics: ', error);
    } finally {
      setIsUpdating(false);
    }
  }

  const generateHeatmapData = () => {
    const startDate = dayjs(`${currentYear}-01-01`).startOf('week');
    const endDate = dayjs(`${currentYear}-12-31`);
    const today = dayjs();
    const months: MonthData[] = [];

    let currentDate = startDate;
    let currentMonthWeeks: StatisticsData[][] = [];

    while (currentDate.isBefore(endDate) || currentDate.isSame(endDate, 'day')) {
      const week: StatisticsData[] = [];
      const weekStart = currentDate.startOf('week');

      for (let i = 0; i < 7; i++) {
        const date = weekStart.add(i, 'day');

        if (date.year() !== currentYear) {
          week.push({ date: '', count: -1 }); // 空のセル
        } else if (date.isAfter(today, 'day')) {
          week.push({ date: '', count: -1 }); // 未来の日付
        } else {
          const dateStr = date.format('YYYY-MM-DD');
          const data = statsData.find(item => item.date === dateStr);
          week.push({
            date: dateStr,
            count: data?.count || 0,
          });
        }
      }

      currentMonthWeeks.push(week);

      const nextWeek = currentDate.add(7, 'day');
      // 年始のDecemberだけ表示しない
      if ((nextWeek.month() !== currentDate.month() || nextWeek.isAfter(endDate))
        && !(nextWeek.year() === currentYear && nextWeek.month() == 0 && currentDate.month() == 11)) {
        months.push({
          monthName: currentDate.format('MMM'),
          weeks: currentMonthWeeks,
        });
        currentMonthWeeks = [];
      }

      currentDate = nextWeek;
    }

    return months;
  };

  const fetchYearStatistics = async (year: number) => {
    try {
      const data = await getYearStatistics(year);
      setStatsData(data);
    } catch (error) {
      console.error('Failed to fetch year statistics:', error);
    }
  }

  const handleYearChange = (direction: 'prev' | 'next') => {
    const newYear = direction === 'prev' ? currentYear - 1 : currentYear + 1;
    setCurrentYear(newYear);
  }

  const getLevel = (count: number): number => {
    if (count === -1) return -1;
    if (count === 0) return 0;
    if (count < 25000) return 1;
    if (count < 50000) return 2;
    if (count < 75000) return 3;
    if (count < 100000) return 4;
    return 5;
  };

  useEffect(() => {
    fetchYearStatistics(currentYear);
  }, [currentYear])

  useEffect(() => {
    setHeatmapData(generateHeatmapData());
  }, [statsData]);

  return (
    <div className='stats-wrapper'>
      <div className='stats-header'>
        <div className="year-selector">
          <button
            onClick={() => handleYearChange('prev')}
            className="year-button"
            aria-label="Previous year"
          >
            <MdNavigateBefore size={20} />
          </button>
          <span className="year-text">{currentYear}</span>
          <button
            onClick={() => handleYearChange('next')}
            className="year-button"
            aria-label="Next year"
            disabled={currentYear >= dayjs().year()}
          >
            <MdNavigateNext size={20} />
          </button>
        </div>
      </div>

      <div className="heatmap-container">
        <button
          onClick={updateStatistics}
          className="refresh-button"
          disabled={isUpdating}
          aria-label="Refresh statistics"
        >
          <IoMdRefresh size={18} className={isUpdating ? 'spinning' : ''} />
        </button>

        <div className="heatmap-content">
          {/* 曜日ラベル */}
          <div className="weekday-labels">
            {weekdays.map((day, index) => (
              <div key={index} className="weekday-label">{day}</div>
            ))}
          </div>

          {/* ヒートマップグリッド */}
          <div className="contribution-grid">
            {heatmapData.map((month, monthIndex) => (
              <div className='contribution-month-wrapper'>
                {/* 月ラベル */}
                <div className='month-label'>
                  {month.monthName}
                </div>

                <div key={monthIndex} className="contribution-month">
                  {month.weeks.map((week, weekIndex) => (
                    <div key={weekIndex} className="contribution-week">
                      {week.map((day, dayIndex) => {
                        const level = getLevel(day.count);
                        return (
                          <div
                            key={`${monthIndex}-${weekIndex}-${dayIndex}`}
                            className={`contribution-day ${level === -1 ? 'empty' : `level-${level}`
                              }`}
                            title={
                              day.date
                                ? `${day.date}: ${day.count.toLocaleString()} keys`
                                : ''
                            }
                          />
                        );
                      })}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* 凡例 */}
        <div className="heatmap-legend">
          <span className="legend-text">Less</span>
          <div className="legend-levels">
            <div className="legend-level level-0" title="No contributions" />
            <div className="legend-level level-1" title="1-24,999 keys" />
            <div className="legend-level level-2" title="25,000-49,999 keys" />
            <div className="legend-level level-3" title="50,000-74,999 keys" />
            <div className="legend-level level-4" title="75,000-99,999 keys" />
            <div className="legend-level level-5" title="100,000+ keys" />
          </div>
          <span className="legend-text">More</span>
        </div>
      </div>
    </div>
  );
};