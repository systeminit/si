import { ChartConfiguration } from "chart.js";
import { IActivitySummaryReplySuccess } from "../sdf/dal/applicationDal";

export function buildActivityData(
  data: IActivitySummaryReplySuccess,
): ChartConfiguration {
  return {
    type: "bar",
    data: {
      labels: data.labels,
      datasets: [
        {
          label: "Change Set Apply",
          borderColor: "#0092B4",
          backgroundColor: "#0092B4",
          lineTension: 0,
          pointRadius: 0,
          fill: false,
          borderWidth: 1,
          data: data.applyData,
        },
        {
          label: "Deploy",
          borderColor: "#B400B1",
          backgroundColor: "#B400B1",
          lineTension: 0,
          pointRadius: 0,
          fill: false,
          borderWidth: 1,
          data: data.deployData,
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
}

export function buildActivityDataSm(
  data: IActivitySummaryReplySuccess,
): ChartConfiguration {
  return {
    type: "bar",
    data: {
      labels: data.labels,
      datasets: [
        {
          label: "Change Set Apply",
          borderColor: "#0092B4",
          backgroundColor: "#0092B4",
          lineTension: 0,
          pointRadius: 0,
          fill: false,
          borderWidth: 1,
          data: data.applyData,
        },
        {
          label: "Deploy",
          borderColor: "#B400B1",
          backgroundColor: "#B400B1",
          lineTension: 0,
          pointRadius: 0,
          fill: false,
          borderWidth: 1,
          data: data.deployData,
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
}
