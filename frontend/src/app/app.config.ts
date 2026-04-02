import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { provideRouter } from '@angular/router';

import { routes } from './app.routes';
import { Identity } from 'spacetimedb';
import { provideSpacetimeDB } from 'spacetimedb/angular';
import { environment } from '../environments/environment';
import { DbConnection, ErrorContext } from '../modules_bindings';

const HOST = environment.SPACETIMEDB_HOST;
const DB_NAME = environment.SPACETIMEDB_DB_NAME;

const onConnect = (_conn: DbConnection, identity: Identity, token: string) => {
  localStorage.setItem('auth_token', token);
  console.log('Connected to SpacetimeDB with identity:', identity.toHexString());
};

const onDisconnect = () => {
  console.log('Disconnected from SpacetimeDB');
  window.location.replace('/');
};

const onConnectError = (_ctx: ErrorContext, err: Error) => {
  console.log(`Error connecting to SpacetimeDB (Host: ${HOST}, Database: ${DB_NAME}):`, err);
};

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes),
    provideSpacetimeDB(
      DbConnection.builder()
        .withUri(HOST)
        .withDatabaseName(DB_NAME)
        .withToken(localStorage.getItem('auth_token') || undefined)
        .onConnect(onConnect)
        .onDisconnect(onDisconnect)
        .onConnectError(onConnectError),
    ),
  ],
};
