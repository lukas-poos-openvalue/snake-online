import { Routes } from '@angular/router';
import { Lobby } from './lobby/lobby';
import { Game } from './game/game';

export const routes: Routes = [
  { path: '', component: Lobby, title: 'Snake - Lobby' },
  { path: 'games/:id', component: Game },
];
