import {autoUpdate, computePosition} from "@floating-ui/dom";

type Options = {
  visible: boolean,
  width: number,
  disposeUpdater: (() => void) | null,
};

/**
 * Makes dropdowns for an element with nested lists.
 *
 * @param elem - The element to traverse and hook onto.
 */
export function hookDropdowns(elem: HTMLElement) {
  const options = {
    visible: false,
    width: elem.clientWidth,
    disposeUpdater: null
  }

  for (const child of elem.children) {
    const htmlChild = child as HTMLElement;

    if (htmlChild !== null)
      registerDropdownsRec(htmlChild, elem, 0, options);
  }
}

function registerDropdownsRec(elem: HTMLElement, reference: HTMLElement, listDepth: number, options: Options) {
  if (elem.tagName === "UL") {
    // register dropdown
    if (!elem.classList.contains("nav-dropdown") && listDepth > 0) {
      elem.classList.add("nav-dropdown");
      // hide by default
      elem.classList.add("hidden");
      createDropButton(elem, reference, options);
    }

    listDepth += 1;
  }

  // iterate through all children
  for (const child of elem.children) {
    const htmlChild = child as HTMLElement;

    if (htmlChild !== null)
      registerDropdownsRec(htmlChild, elem, listDepth, options);
  }
}

function createDropButton(elem: HTMLElement, reference: HTMLElement, baseOptions: Options) {
  const options = Object.assign({}, baseOptions);

  reference.classList.add("nav-dropdown-reference");

  // create the actual drop button
  const button = document.createElement("span");
  button.classList.add("nav-dropdown-button");
  button.classList.add("material-symbols-outlined");
  button.textContent = "arrow_drop_down";

  button.addEventListener("click", () => {
    options.visible = !options.visible;

    if (options.visible) {
      // create updater
      button.classList.add("toggled");
      elem.classList.remove("hidden");
      elem.classList.add("drop");
      elem.classList.remove("undrop");
      options.disposeUpdater = autoUpdate(elem, reference, () => {
        update(elem, reference, options);
      });
    } else {
      button.classList.remove("toggled");
      elem.classList.add("undrop");
      elem.classList.remove("drop");
      const disposer = () => {
        elem.classList.add("hidden");
        if (options.disposeUpdater !== null) {
          options.disposeUpdater();
        }
        elem.removeEventListener("animationend", disposer);
      };
      elem.addEventListener("animationend", disposer);
    }
  });

  // create a container for the content
  const content = document.createElement("div");
  content.classList.add("nav-dropdown-content");

  for (const childElem of reference.children) {
    content.appendChild(childElem);
  }

  reference.appendChild(content);
  reference.appendChild(button);
}

function update(elem: HTMLElement, reference: HTMLElement, options: Options) {
  computePosition(reference, elem).then(({x, y}) => {
    Object.assign(elem.style, {
      width: `calc(${options.width}px - 2em)`,
      height: `${elem.scrollHeight}px`,
      left: `${x}px`,
      top: `${y}px`,
    });
  });
}
