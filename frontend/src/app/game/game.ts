import { Component, computed, effect, HostListener, inject } from '@angular/core';
import { injectReducer, injectSpacetimeDB, injectTable } from 'spacetimedb/angular';
import { reducers, tables } from '../../modules_bindings';
import { Direction } from '../../modules_bindings/types';
import { Router } from '@angular/router';

@Component({
  selector: 'app-game',
  imports: [],
  templateUrl: './game.html',
  styleUrl: './game.css',
})
export class Game {
  readonly conn = injectSpacetimeDB();
  readonly #router = inject(Router);

  readonly #activeGame = injectTable(tables.active_game);
  readonly game = computed(() => this.#activeGame().rows[0]);
  readonly #isUserActive = computed(
    () =>
      this.game()
        .players.filter((p) => p.isActive)
        .filter((p) => this.conn().identity?.equals(p.identity) ?? false).length === 1,
  );
  readonly isUserOwner = computed(
    () =>
      this.game()
        .players.filter((p) => p.isOwner)
        .filter((p) => this.conn().identity?.equals(p.identity) ?? false).length === 1,
  );
  readonly #setNextDirection = injectReducer(reducers.setNextDirection);
  readonly #restartGame = injectReducer(reducers.restartGame);
  readonly #closeGame = injectReducer(reducers.closeGame);

  constructor() {
    effect(() => {
      const activeGameRow = this.#activeGame();
      if (activeGameRow.isLoading) {
        return;
      }
      if (activeGameRow.rows.length === 0 || activeGameRow.rows[0].state.tag === 'Closed') {
        this.#router.navigate(['/'], { replaceUrl: true });
      }
    });
  }

  @HostListener('document:keyup', ['$event'])
  handleUserInput(event: KeyboardEvent) {
    if (!this.#isUserActive()) {
      return;
    }

    let direction: Direction | undefined = undefined;
    if (event.key === 'ArrowUp' || event.key === 'k' || event.key === 'w') {
      direction = Direction.Up;
    } else if (event.key === 'ArrowDown' || event.key === 'j' || event.key === 's') {
      direction = Direction.Down;
    } else if (event.key === 'ArrowLeft' || event.key === 'h' || event.key === 'a') {
      direction = Direction.Left;
    } else if (event.key === 'ArrowRight' || event.key === 'l' || event.key === 'd') {
      direction = Direction.Right;
    }

    if (!direction) {
      return;
    }

    this.#setNextDirection({ gameId: this.game().gameId, direction });
  }

  restartGame() {
    this.#restartGame({ gameId: this.game().gameId });
  }

  closeGame() {
    this.#closeGame({ gameId: this.game().gameId });
  }
}
