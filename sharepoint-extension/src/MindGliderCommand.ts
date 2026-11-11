import { Dialog } from '@microsoft/sp-dialog';
import {
  BaseListViewCommandSet,
  IListViewCommandSetExecuteEventParameters,
  RowAccessor
} from '@microsoft/sp-listview-extensibility';
import { desktopLink, isMap } from './desktopLink';

interface DriveItem {
  id: string;
  name: string;
  parentReference: { driveId: string };
  file?: object;
}

export default class MindGliderCommand extends BaseListViewCommandSet<Record<string, never>> {
  private selection = '';
  private link: string | undefined;
  private generation = 0;

  public onInit(): Promise<void> {
    this.tryGetCommand('OPEN_MINDGLIDER').visible = false;
    this.context.listView.listViewStateChangedEvent.add(this, this.selectionChanged);
    this.selectionChanged();
    return Promise.resolve();
  }

  public onDispose(): void {
    this.generation++;
    this.context.listView.listViewStateChangedEvent.remove(this, this.selectionChanged);
  }

  public onExecute(event: IListViewCommandSetExecuteEventParameters): void {
    if (event.itemId === 'OPEN_MINDGLIDER' &&
        this.link && this.rowId(event.selectedRows) === this.selection) {
      window.location.assign(this.link);
    }
  }

  private rowId(rows: readonly RowAccessor[] | undefined): string {
    if (rows?.length !== 1 || !isMap(rows[0].getValueByName('FileLeafRef'))) return '';
    const id = String(rows[0].getValueByName('ID'));
    return /^\d+$/.test(id) ? id : '';
  }

  private selectionChanged = (): void => {
    const id = this.rowId(this.context.listView.selectedRows);
    if (id === this.selection) return;
    this.selection = id;
    const generation = ++this.generation;
    this.link = undefined;
    this.tryGetCommand('OPEN_MINDGLIDER').visible = false;
    this.raiseOnChange();
    if (id) this.resolve(id, generation).catch(() => {
      if (this.generation !== generation) return;
      // Allow the user to retry by selecting the row again.
      this.selection = '';
      void Dialog.alert('MindGlider could not resolve the SharePoint file. Check the app permissions.');
    });
  };

  private async resolve(id: string, generation: number): Promise<void> {
    const page = this.context.pageContext;
    if (!page.list || !page.aadInfo) throw new Error('Missing library context');
    const host = new URL(page.web.absoluteUrl).hostname;
    const site = [host, page.site.id.toString(), page.web.id.toString()].join(',');
    const graph = await this.context.msGraphClientFactory.getClient('3');
    const result: DriveItem = await graph
      .api(`/sites/${site}/lists/${page.list.id.toString()}/items/${id}/driveItem`)
      .select('id,name,file,parentReference')
      .get();
    if (generation !== this.generation || !result.file || !isMap(result.name)) return;
    this.link = desktopLink(page.aadInfo.tenantId.toString(), result.parentReference.driveId, result.id);
    this.tryGetCommand('OPEN_MINDGLIDER').visible = true;
    this.raiseOnChange();
  }
}
