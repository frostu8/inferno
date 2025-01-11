type Options = {
  visible: boolean,
  width: number
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
    zIndex: 1
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
      Object.assign(elem.style, { height: `${elem.scrollHeight + 8}px` });
      elem.classList.add("nav-dropdown");

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
  const options = { ...baseOptions };

  reference.classList.add("nav-dropdown-reference");

  // create the actual drop button
  const button = document.createElement("span");
  button.classList.add("nav-dropdown-button");
  button.classList.add("material-symbols-outlined");
  button.textContent = "arrow_drop_down";

  button.addEventListener("click", () => {
    options.visible = !options.visible;

    if (options.visible) {
      button.classList.add("toggled");

      elem.classList.add("drop");
    } else {
      button.classList.remove("toggled");

      elem.classList.remove("drop");
      elem.classList.add("undrop");

      const disposer = () => {
        elem.classList.remove("undrop");
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
