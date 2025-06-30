import { createMemoryHistory, createRouter } from "vue-router";
import PizzaList from "./components/ListByVotes.vue";
import NewPizzaSubmit from "./components/NewPizzaSubmit.vue";
import Account from "./components/Account.vue";
import InfoPage from "./components/InfoPage.vue";

import "./assets/main.css";

const routes = [
  { path: "/", component: PizzaList },
  { path: "/pizza-submission", component: NewPizzaSubmit },
  { path: "/account", component: Account },
  { path: "/about", component: InfoPage }
];

export const router = createRouter({
  history: createMemoryHistory(),
  routes
});
