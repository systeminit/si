import axios from "axios";
import { Context } from "../context";

interface GreetResponse {
  greeting: string;
}

export async function greet(ctx: Context, name: string): Promise<string> {
  try {
    const { data } = await ctx.httpClient.post<GreetResponse>("/greet", {
      name,
    });
    return data.greeting;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.log("error message:", error.message);
      throw error;
    } else {
      console.log("unexpected error:", error);
      throw error;
    }
  }
}
