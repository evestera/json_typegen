import hnStory from "./hnStory.json";
import magicCardList from "./magicCardList.json";
import steamAppNews from "./steamAppNews.json";
import worldBankIndicator from "./worldBankIndicator.json";
import zalandoArticle from "./zalandoArticle.json";

export interface Example {
  id: string;
  name: string;
  json: string;
  typeName: string;
  propertyNameFormat?: string;
}

export const examples: Example[] = [
  {
    id: "hnStory",
    name: "Hacker News Story",
    json: JSON.stringify(hnStory, undefined, 2),
    typeName: "Story",
  },
  {
    id: "magicCardList",
    name: "List of Magic cards",
    json: JSON.stringify(magicCardList, undefined, 2),
    typeName: "Cards",
    propertyNameFormat: "camelCase",
  },
  {
    id: "steamAppNews",
    name: "Steam App News",
    json: JSON.stringify(steamAppNews, undefined, 2),
    typeName: "AppnewsWrapper",
    propertyNameFormat: "snake_case",
  },
  {
    id: "worldBankIndicator",
    name: "World Bank Indicator",
    json: JSON.stringify(worldBankIndicator, undefined, 2),
    typeName: "Indicator",
    propertyNameFormat: "snake_case",
  },
  {
    id: "zalandoArticle",
    name: "Zalando Article",
    json: JSON.stringify(zalandoArticle, undefined, 2),
    typeName: "Article",
    propertyNameFormat: "camelCase",
  },
];
