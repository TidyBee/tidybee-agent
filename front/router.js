import Vue from 'vue';
import VueRouter from 'vue-router';

import SystemStatisticsView from './views/systemStatisticsView.vue';
import ConfigurationPanel from './views/configurationPanel.vue';

Vue.use(VueRouter);

const routes = [
    { path: '/systemStatistics', component: SystemStatisticsView },
    { path: '/configurationPanel', component: ConfigurationPanel },
    { path: '/about', component: AnotherView },
    { path: '/config', component: AnotherView },
    { path: '/contact', component: AnotherView },
];

const router = new VueRouter({
    mode: 'history',
    routes,
});

export default router;
