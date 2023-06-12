import Vue from 'vue';
import App from './App.vue';
import router from './router';

new Vue({
    router, // Add the router instance to the Vue app
    render: (h) => h(App),
}).$mount('#app');
