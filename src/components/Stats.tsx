import dayjs from 'dayjs';

import './Stats.css';

type statsData = {
  date: string,
  count: number,
}

export default function Stats() {
  const sampleData = [
    { date: '2025-01-01', count: 1 },
    { date: '2025-01-02', count: 25001 },
    { date: '2025-01-03', count: 50001 },
    { date: '2025-01-04', count: 75001 },
    { date: '2025-01-05', count: 100001 },
    { date: '2025-01-05', count: 125001 },
  ];


  const today = dayjs();
  const startDate = dayjs('2025-01-01');

  const generateHeatmapData = () => {
    const weeks = [];
    let currentDate = startDate;

    while (currentDate.isBefore(today) || currentDate.isSame(today, 'day')) {
      const week = [];
      for (let i = 0; i < 7; i++) {
        const dateStr = currentDate.format('YYYY-MM-DD');
        const data = sampleData.find(item => item.date === dateStr);

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