using System.Collections.Specialized;
using System.Windows;
using CapivaraBooster.ViewModels;

namespace CapivaraBooster;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
    }

    public MainWindow(MainViewModel viewModel) : this()
    {
        DataContext = viewModel;

        if (viewModel.LogItems is INotifyCollectionChanged notify)
        {
            notify.CollectionChanged += (_, _) =>
            {
                Dispatcher.InvokeAsync(() =>
                {
                    if (ConsoleListBox.Items.Count > 0)
                    {
                        ConsoleListBox.ScrollIntoView(ConsoleListBox.Items[^1]);
                    }
                }, System.Windows.Threading.DispatcherPriority.Background);
            };
        }
    }
}
