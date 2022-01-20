import { Qualification } from "@/api/sdf/dal/qualification";

export function listQualifications(componentId: number): Array<Qualification> {
  console.log(componentId);
  const qualification = {
    name: "ilikemybutt",
    title: "I LIKE MY BUTT",
    link: "https://bit.ly/3qHuTNh",
    description: "description of i like my butt",
    result: {
      success: false,
      errors: [{ message: "there's a snake in my boot!" }],
    },
  };
  return [qualification];
}
