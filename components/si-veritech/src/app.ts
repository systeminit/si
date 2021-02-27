import express from "express";
import expressWs from "express-ws";
import morgan from "morgan";
import controller from "./controllers";

export const app = expressWs(express()).app;
app.use(express.json());
app.use(morgan("dev", { skip: () => process.env.NODE_ENV == "test" }));

app.post("/inferProperties", controller.inferProperties);
