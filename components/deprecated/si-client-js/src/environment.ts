import dotenv from "dotenv";

dotenv.config();

interface Environment {
  apollo: {
    authorization: string;
  }
}

export const env: Environment = {
  apollo: {
    authorization: process.env.APOLLO_AUTHORIZATION || "",
  },
};
