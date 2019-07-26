import { Model } from 'objection';
import Knex from 'knex';
import { environment } from '@/environment';
import knexConfig from '../knexfile';

const config = knexConfig[environment.node_env];
export const knex = Knex(config);
Model.knex(knex);
export default Model;
