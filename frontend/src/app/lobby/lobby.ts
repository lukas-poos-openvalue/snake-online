import { Component, linkedSignal, signal } from '@angular/core';
import { injectReducer, injectTable } from 'spacetimedb/angular';
import { reducers, tables } from '../../modules_bindings';

@Component({
  selector: 'app-lobby',
  imports: [],
  templateUrl: './lobby.html',
  styleUrl: './lobby.css',
})
export class Lobby {
  readonly joinableGames = injectTable(tables.joinable_game);
  readonly currentUser = injectTable(tables.my_user);
  readonly newGameName = linkedSignal(() =>
    this.currentUser().isLoading
      ? "Annonymous's Game"
      : `${this.currentUser().rows[0].name ?? 'Annonymous'}'s Game`,
  );

  readonly #createGame = injectReducer(reducers.createGame);
  readonly #joinGame = injectReducer(reducers.joinGame);

  createGame() {
    if (this.newGameName() === '') {
      return;
    }
    this.#createGame({ name: this.newGameName() });
  }

  joinGame(gameId: bigint) {
    this.#joinGame({ gameId });
  }
}
