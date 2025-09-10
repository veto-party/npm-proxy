import axios from 'axios';

export const client = axios.create({
    baseURL: import.meta.env.DEV ? 'http://localhost:5000' : '/'
});