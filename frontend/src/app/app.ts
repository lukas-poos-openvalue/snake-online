import { Component, effect, inject, linkedSignal, Signal, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterOutlet } from '@angular/router';
import { injectReducer, injectSpacetimeDB, injectTable } from 'spacetimedb/angular';
import { reducers, tables } from '../modules_bindings';
import { Game } from './game/game';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  templateUrl: './app.html',
})
export class App {
  readonly #router = inject(Router);
  readonly #activatedRoute = inject(ActivatedRoute);
  readonly #myUser = injectTable(tables.my_user);
  readonly #myGame = injectTable(tables.my_game);
  readonly #setUserName = injectReducer(reducers.setUserName);

  readonly conn = injectSpacetimeDB();
  readonly userName = linkedSignal(() => this.#myUser().rows[0]?.name);
  readonly #userNameUpdates = debouncedSignal(this.userName, 100);

  constructor() {
    effect(() => {
      const name = this.#userNameUpdates();
      if (name) {
        this.#setUserName({ name });
      }
    });
    effect(() => {
      const myGame = this.#myGame().rows[0];
      if (myGame && this.#activatedRoute.component !== Game) {
        this.#router.navigate(['/games', myGame.gameId]);
      }
    });
  }
}

function debouncedSignal<T>(sourceSignal: Signal<T>, debounceTimeInMs = 0): Signal<T> {
  const debounceSignal = signal(sourceSignal());
  effect(
    (onCleanup) => {
      const value = sourceSignal();
      const timeout = setTimeout(() => debounceSignal.set(value), debounceTimeInMs);
      onCleanup(() => clearTimeout(timeout));
    },
    { allowSignalWrites: true },
  );
  return debounceSignal;
}
