import { Component, effect, inject, OnDestroy } from '@angular/core';
import { ActivatedRoute, Router, RouterOutlet } from '@angular/router';
import { injectReducer, injectSpacetimeDB, injectTable } from 'spacetimedb/angular';
import { reducers, tables } from '../modules_bindings';
import { Game } from './game/game';
import { debounceTime, filter, Subject, Subscription } from 'rxjs';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App implements OnDestroy {
  readonly router = inject(Router);
  readonly activatedRoute = inject(ActivatedRoute);

  readonly conn = injectSpacetimeDB();
  readonly myUser = injectTable(tables.my_user);
  readonly myGame = injectTable(tables.my_game);
  readonly setUserName = injectReducer(reducers.setUserName);

  readonly userNameUpdates$ = new Subject<string>();
  readonly userNameUpdatesSub: Subscription;

  constructor() {
    // Handle inputs to the user name input
    this.userNameUpdatesSub = this.userNameUpdates$
      .pipe(
        filter((name) => name?.trim() !== ''),
        debounceTime(200),
      )
      .subscribe((name) => this.setUserName({ name }));

    // Redirect to active game if present
    effect(() => {
      const myGame = this.myGame().rows[0];
      if (myGame && this.activatedRoute.component !== Game) {
        this.router.navigate(['/games', myGame.gameId]);
      }
    });
  }

  ngOnDestroy(): void {
    this.userNameUpdatesSub?.unsubscribe();
  }
}
