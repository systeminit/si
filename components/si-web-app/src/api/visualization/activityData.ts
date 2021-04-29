import { ChartConfiguration } from "chart.js";

function labels(): string[] {
  // return ["s", "s", "m", "t", "w", "t", "f"];
  // return ["S", "S", "M", "T", "W", "T", "F"];
  return ["Sa", "Su", "Mo", "Tu", "We", "Th", "Fr"];
}

function applydata(): number[] {
  return [3, 2, 4, 3, 3, 1, 5];
}

function deployData(): number[] {
  return [1, 1, 0, 6, 5, 1, 2];
}

export const activityData: ChartConfiguration = {
  type: "line",
  data: {
    labels: labels(),
    datasets: [
      {
        label: "Change Set Apply",
        borderColor: "#0092B4",
        backgroundColor: "#0092B4",
        lineTension: 0,
        pointRadius: 0,
        fill: false,
        borderWidth: 1,
        data: applydata(),
      },
      {
        label: "Deploy",
        borderColor: "#B400B1",
        backgroundColor: "#B400B1",
        lineTension: 0,
        pointRadius: 0,
        fill: false,
        borderWidth: 1,
        data: deployData(),
      },
    ],
  },
  options: {
    responsive: true,
    maintainAspectRatio: false,
    showLines: true,
    tooltips: {
      enabled: true,
    },
    legend: {
      display: false,
    },
    layout: {
      padding: 10,
    },
    scales: {
      display: true,
      xAxes: [
        {
          display: true,
          ticks: {
            display: true,
            fontSize: 9,
            maxRotation: 0,
            minRotation: 0,
          },
        },
      ],
      yAxes: [
        {
          display: false,
        },
      ],
    },
  },
};

export const activityDataSm: ChartConfiguration = {
  type: "line",
  data: {
    labels: labels(),
    datasets: [
      {
        label: "Change Set Apply",
        borderColor: "#0092B4",
        backgroundColor: "#0092B4",
        lineTension: 0,
        pointRadius: 0,
        fill: false,
        borderWidth: 1,
        data: applydata(),
      },
      {
        label: "Deploy",
        borderColor: "#B400B1",
        backgroundColor: "#B400B1",
        lineTension: 0,
        pointRadius: 0,
        fill: false,
        borderWidth: 1,
        data: deployData(),
      },
    ],
  },
  options: {
    responsive: true,
    maintainAspectRatio: false,
    showLines: true,
    tooltips: {
      enabled: false,
    },
    legend: {
      display: false,
    },
    layout: {
      padding: {
        top: 0,
        bottom: 0,
        left: 4,
        right: 4,
      },
    },
    scales: {
      display: true,
      xAxes: [
        {
          display: true,
          ticks: {
            display: false,
            fontSize: 9,
            maxRotation: 0,
            minRotation: 0,
          },
        },
      ],
      yAxes: [
        {
          display: false,
        },
      ],
    },
  },
};
